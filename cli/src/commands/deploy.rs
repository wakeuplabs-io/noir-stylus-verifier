use crate::{
    infrastructure::{console::progress::style_spinner, downloader::github},
    AppContext,
};
use colored::*;
use indicatif::ProgressBar;
use std::{env, path::PathBuf};

pub(crate) struct DeployCommand {}

impl DeployCommand {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        circuit: &str,
        zk_flavor: bool,
        rpc_url: &str, 
        private_key: &str,
        verifier_address: Option<String>, // TODO: option, if not provided take constants from provided
    ) -> Result<(), Box<dyn std::error::Error>> {
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
            &format!("⏳ Deploying {}...", root.display()),
        );

        // TODO: export the vk and circuit json using bb and move to `contracts/assets`

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
