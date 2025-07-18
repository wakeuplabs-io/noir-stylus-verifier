use crate::{
    config::requirements::{
        SystemRequirementsChecker, TSystemRequirementsChecker, CARGO_STYLUS_REQUIREMENT,
    },
    infrastructure::{console::progress::style_spinner, downloader::github, system::System},
    AppContext,
};
use colored::*;
use indicatif::ProgressBar;
use std::{env, path::PathBuf};

pub(crate) struct DeployCommand {
    system_requirements_checker: SystemRequirementsChecker,
}

impl DeployCommand {
    pub(crate) fn new() -> Self {
        Self {
            system_requirements_checker: SystemRequirementsChecker::new(),
        }
    }

    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        rpc_url: &str,
        private_key: &str,
        verifier_address: Option<String>, // TODO: option, if not provided take constants from provided
        zk_flavor: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.system_requirements_checker
            .check(vec![CARGO_STYLUS_REQUIREMENT])?;

            // TODO: circuit or root
        let mut root = PathBuf::from(".");
        if !root.is_absolute() {
            root = env::current_dir()?.join(root)
        }


        let create_spinner = style_spinner(
            ProgressBar::new_spinner(),
            &format!("⏳ Deploying {}...", root.display()),
        );

        // TODO: deploy the verifier to the blockchain using cargo-stylus

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
