use crate::{
    config::requirements::{
        SystemRequirementsChecker, TSystemRequirementsChecker, BB_REQUIREMENT, BB_UP_REQUIREMENT,
        NOIRUP_REQUIREMENT, NOIR_REQUIREMENT,
    },
    infrastructure::{
        codegen::verifier_generator::VerifierGenerator,
        console::progress::style_spinner,
        system::{System, TSystem},
    },
    AppContext,
};
use colored::*;
use indicatif::ProgressBar;
use std::{env, path::PathBuf, process::Command};

pub(crate) struct GenerateCommand {
    system: System,
    system_requirements_checker: SystemRequirementsChecker,
    verifier_generator: VerifierGenerator,
}

impl GenerateCommand {
    pub(crate) fn new() -> Self {
        Self {
            system: System::new(),
            system_requirements_checker: SystemRequirementsChecker::new(),
            verifier_generator: VerifierGenerator::new(),
        }
    }

    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        circuit: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.system_requirements_checker
            .check(vec![BB_UP_REQUIREMENT, NOIRUP_REQUIREMENT])?;

        let root = if circuit.is_some() {
            PathBuf::from(circuit.unwrap())
        } else {
            env::current_dir()?
        };

        // verify we are in a circuit directory. TODO: properly handle workspaces, same for binaries
        if !root.join("Nargo.toml").exists() {
            return Err(format!("Directory {} does not contain a circuit", root.display()).into());
        }
        let package_name = self.read_package_name(root.clone())?;

        let spinner = style_spinner(
            ProgressBar::new_spinner(),
            &format!("⏳ Generating {}...", root.display()),
        );

        let contracts_root = root.join("contracts");
        self.system.ensure_dir(&contracts_root)?;

        // set noir version
        spinner.set_message(format!(
            "Setting noir version to {}...",
            NOIR_REQUIREMENT.required_version
        ));
        self.system.execute_command(
            Command::new("noirup")
                .arg("-v")
                .arg(NOIR_REQUIREMENT.required_version),
        )?;

        // set bb version
        spinner.set_message(format!(
            "Setting bb version to {}...",
            BB_REQUIREMENT.required_version
        ));
        self.system.execute_command(
            Command::new("bbup")
                .arg("-v")
                .arg(BB_REQUIREMENT.required_version),
        )?;

        spinner.set_message("Compiling circuit...");
        self.system
            .execute_command(Command::new("nargo").arg("compile").current_dir(&root))?;

        spinner.set_message("Writing vk...");
        self.system.execute_command(
            Command::new("bb")
                .arg("write_vk")
                .arg("--oracle_hash")
                .arg("keccak")
                .arg("-o")
                .arg("target")
                .arg("-b")
                .arg(root.join("target").join(format!("{}.json", package_name)))
                .current_dir(&root),
        )?;

        // generate verifier contract
        let project_files = self.verifier_generator.generate_verifier_contract(
            &root.join("target").join(format!("{}.json", package_name)),
            &root.join("target").join("vk"),
        )?;
        for file in project_files {
            spinner.set_message(format!("Writing {}", file.path));
            self.system
                .write_file(&contracts_root.join(file.path), file.content)?;
        }

        spinner.finish_with_message(format!(
            "{} Generated at {}\n",
            "✅ Success!".green(),
            contracts_root.display()
        ));

        // print instructions ========================================

        println!(
            "\n {title}\n\n  - {bin} {check_cmd}: Checks the verifier contract.\n  - {bin} {deploy_cmd}: Deploys the verifier to the blockchain.\n",
            title = "What's Next?".bright_white().bold(),
            bin = env!("CARGO_BIN_NAME").blue(),
            check_cmd = "check".blue(),
            deploy_cmd = "deploy".blue() 
        );

        Ok(())
    }

    fn read_package_name(&self, root: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
        let package_toml = self.system.read_file_str(&root.join("Nargo.toml"))?;
        let package_name = package_toml
            .split("name = ")
            .nth(1)
            .unwrap()
            .split("\n")
            .nth(0)
            .unwrap()
            .split(" ")
            .nth(0)
            .unwrap()
            .replace("\"", "")
            .to_string();

        Ok(package_name)
    }
}
