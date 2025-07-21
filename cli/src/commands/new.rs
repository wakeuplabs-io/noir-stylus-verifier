use crate::{
    infrastructure::{
        codegen::{Codegen, TCodegen},
        progress::create_spinner,
        system::{System, TSystem},
    },
    AppContext,
};
use colored::*;
use std::{env, path::PathBuf};

pub(crate) struct NewCommand {
    system: Box<dyn TSystem>,
    codegen: Box<dyn TCodegen>,
}

impl Default for NewCommand {
    fn default() -> Self {
        Self {
            system: Box::new(System::default()),
            codegen: Box::new(Codegen::default()),
        }
    }
}

impl NewCommand {
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

        // generate project
        let project_files = self.codegen.generate_project(name)?;
        for file in project_files {
            spinner.set_message(format!("Writing {}", file.path));
            self.system
                .write_file(&root.join(file.path), file.content)?;
        }

        spinner.finish_with_message(format!(
            "{} Created {name} at {}\n",
            "✅ Success!".green(),
            root.display()
        ));

        // print instructions ========================================

        println!(
            "\n {title}\n\n  - {bin} {generate_cmd}: Generates a new verifier contract.\n  - {bin} {check_cmd}: Checks the verifier contract.\n  - {bin} {deploy_cmd}: Deploys the verifier to the blockchain.\n",
            title = "What's Next?".bright_white().bold(),
            bin = env!("CARGO_BIN_NAME").blue(),
            generate_cmd = "generate".blue(),
            check_cmd = "check".blue(),
            deploy_cmd = "deploy".blue(),
        );

        Ok(())
    }
}
