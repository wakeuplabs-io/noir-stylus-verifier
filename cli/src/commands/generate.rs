use crate::{
    config::requirements::{
        SystemRequirementsChecker, TSystemRequirementsChecker, BB_UP_REQUIREMENT,
        NOIRUP_REQUIREMENT,
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
        circuit: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.system_requirements_checker
            .check(vec![BB_UP_REQUIREMENT, NOIRUP_REQUIREMENT])?;

        let mut root = PathBuf::from(&circuit);
        if !root.is_absolute() {
            root = env::current_dir()?.join(root)
        }

        if root.exists() {
            return Err(format!("Directory already exists: {}", root.display()).into());
        } else {
            std::fs::create_dir_all(&root)?;
        }

        let create_spinner = style_spinner(
            ProgressBar::new_spinner(),
            &format!("⏳ Generating {}...", root.display()),
        );

        // // TODO: check if contracts folder exists, otherwise ask where to create it
        // if !root.join("contracts").exists() {
        //     let contracts_path = ask_for_contracts_path();
        //     std::fs::create_dir_all(&contracts_path)?;
        // }

        // use noirup and bbup to ensure versions required
        self.system.execute_command(
            Command::new("noirup")
                .arg("-v")
                .arg(NOIRUP_REQUIREMENT.required_version),
        )?;
        self.system.execute_command(
            Command::new("bbup")
                .arg("-v")
                .arg(BB_UP_REQUIREMENT.required_version),
        )?;

        // TODO: export the vk and circuit json using bb and move to `contracts/assets`
        self.system.execute_command(
            Command::new("bb")
                .arg("export-vk")
                .arg(&root.join("contracts/assets/vk.json")),
        )?;
        self.system.execute_command(
            Command::new("bb")
                .arg("export-vk")
                .arg(&root.join("contracts/assets/vk.json")),
        )?;

        // TODO: adapt verifier contract prototype based on circuit inputs

        create_spinner.finish_with_message(format!(
            "{} Deployed at {}\n",
            "✅ Success!".green(),
            root.display() // TODO: address of verifier
        ));

        // print instructions ========================================

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
