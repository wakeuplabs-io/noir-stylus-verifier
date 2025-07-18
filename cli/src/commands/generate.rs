use crate::{
    config::requirements::{
        SystemRequirementsChecker, TSystemRequirementsChecker, BB_REQUIREMENT, BB_UP_REQUIREMENT,
        NOIRUP_REQUIREMENT, NOIR_REQUIREMENT,
    },
    infrastructure::{
        console::progress::style_spinner,
        system::{System, TSystem},
    },
    AppContext,
};
use colored::*;
use indicatif::ProgressBar;
use std::{env, path::PathBuf, process::Command};

struct CircuitInputs {
    name: String,
    visibility: String,
}

pub(crate) struct GenerateCommand {
    system: System,
    system_requirements_checker: SystemRequirementsChecker,
}

impl GenerateCommand {
    pub(crate) fn new() -> Self {
        Self {
            system: System::new(),
            system_requirements_checker: SystemRequirementsChecker::new(),
        }
    }

    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        circuit_root: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.system_requirements_checker
            .check(vec![BB_UP_REQUIREMENT, NOIRUP_REQUIREMENT])?;

        let root = if circuit_root.is_some() {
            PathBuf::from(circuit_root.unwrap())
        } else {
            env::current_dir()?
        };

        // verify we are in a circuit directory
        if !root.join("Nargo.toml").exists() {
            return Err(format!("Directory {} does not contain a circuit", root.display()).into());
        }

        let spinner = style_spinner(
            ProgressBar::new_spinner(),
            &format!("⏳ Generating {}...", root.display()),
        );

        spinner.set_message("Checking noir version...");

        let contracts_root = root.join("contracts");
        if !contracts_root.exists() {
            std::fs::create_dir_all(&contracts_root)?;
        }

        // use noirup and bbup to ensure versions required
        spinner.set_message("Checking noir version...");
        self.system.execute_command(
            Command::new("noirup")
                .arg("-v")
                .arg(NOIR_REQUIREMENT.required_version),
        )?;
        spinner.set_message("Checking bb version...");
        self.system.execute_command(
            Command::new("bbup")
                .arg("-v")
                .arg(BB_REQUIREMENT.required_version),
        )?;

        // TODO: export the vk using bb and read bytes from it.
        // compile circuit
        self.system
            .execute_command(Command::new("nargo").arg("compile").current_dir(&root))?;

        // parse package toml to find package name
        let package_toml = std::fs::read_to_string(root.join("Nargo.toml"))?;
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
            .to_string();

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
        let vk_bytes = std::fs::read(root.join("target").join("vk"))?;

        // TODO: adapt verifier contract prototype based on circuit inputs
        // read circuit json
        let circuit_json_str = std::fs::read_to_string(root.join("target/hello_world.json"))?;
        let circuit_json: serde_json::Value = serde_json::from_str(&circuit_json_str)?;
        let circuit_abi = circuit_json["abi"].clone();
        let circuit_inputs = circuit_abi["parameters"].clone();
        let circuit_inputs = circuit_inputs.as_array().unwrap();
        let circuit_inputs: Vec<CircuitInputs> = circuit_inputs
            .iter()
            .map(|input| CircuitInputs {
                name: input["name"].as_str().unwrap().to_string(),
                visibility: input["visibility"].as_str().unwrap().to_string(),
            })
            .collect::<Vec<CircuitInputs>>();

        // TODO: extract inputs, and generate verifier contract
        // TODO: ensure folders exist and add overwrite feature
        spinner.set_message("Writing project...");
        if !contracts_root.join("src").exists() {
            std::fs::create_dir_all(contracts_root.join("src"))?;
        }
        std::fs::write(contracts_root.join("src/main.rs"), MAIN_RS)?;
        std::fs::write(
            contracts_root.join("src/lib.rs"),
            generate_verifier_contract(circuit_inputs, circuit_json_str, vk_bytes),
        )?;
        std::fs::write(contracts_root.join("Cargo.toml"), CARGO_TOML)?;
        std::fs::write(contracts_root.join(".gitignore"), GITIGNORE)?;
        std::fs::write(contracts_root.join("rust-toolchain.toml"), RUST_TOOLCHAIN)?;
        std::fs::write(contracts_root.join("src/main.rs"), MAIN_RS)?;

        spinner.finish_with_message(format!(
            "{} Generated at {}\n",
            "✅ Success!".green(),
            root.display() // TODO: address of verifier
        ));

        // print instructions ========================================

        // TODO: check and deploy
        println!(
            "\n {title}\n\n  - {bin} {build_cmd}: Builds the stylus compatible wasm.\n  - {bin} {deploy_cmd}: Deploys the verifier to the blockchain.\n",
            title = "What's Next?".bright_white().bold(),
            bin = env!("CARGO_BIN_NAME").blue(),
            build_cmd = "build".blue(),
            deploy_cmd = "deploy".blue() // TODO: expand upon new commands
        );

        Ok(())
    }
}
