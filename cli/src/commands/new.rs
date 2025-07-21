use crate::{
    infrastructure::{console::progress::create_spinner, downloader::github},
    AppContext,
};
use colored::*;
use std::{env, path::PathBuf};

pub(crate) struct NewCommand {}

impl NewCommand {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut root = PathBuf::from(&name);
        if !root.is_absolute() {
            root = env::current_dir()?.join(root)
        }

        if root.exists() {
            return Err(format!("Directory already exists: {}", root.display()).into());
        } else {
            std::fs::create_dir_all(&root)?;
        }

        let spinner = create_spinner(&format!("⏳ Creating {name} at {}...", root.display()));

        // Download hello world example
        // TODO: update this paths once repo is public and we have a release
        github::download_zipped_asset("wakeuplabs-io/op-ruaas", "v1.0.1", "infra-aws", &root)
            .await?;

        spinner.finish_with_message(format!(
            "{} Created {name} at {}\n",
            "✅ Success!".green(),
            root.display()
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
