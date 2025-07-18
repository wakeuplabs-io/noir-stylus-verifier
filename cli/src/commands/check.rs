use crate::{
    config::requirements::{
        SystemRequirementsChecker, TSystemRequirementsChecker, CARGO_STYLUS_REQUIREMENT,
    },
    infrastructure::{
        console::progress::style_spinner, stylus::Stylus,
    },
    AppContext,
};
use colored::*;
use indicatif::ProgressBar;
use std::{env, path::PathBuf};

pub(crate) struct CheckCommand {
    system_requirements_checker: SystemRequirementsChecker,
    stylus: Stylus,
}

impl CheckCommand {
    pub(crate) fn new() -> Self {
        Self {
            system_requirements_checker: SystemRequirementsChecker::new(),
            stylus: Stylus::new(),
        }
    }

    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        circuit: Option<String>,
        rpc_url: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.system_requirements_checker
            .check(vec![CARGO_STYLUS_REQUIREMENT])?;

        let root = if circuit.is_some() {
            PathBuf::from(circuit.unwrap())
        } else {
            env::current_dir()?
        };
        let contracts_root = root.join("contracts");

        // verify we are in a circuit directory.
        if !root.join("Nargo.toml").exists() {
            return Err(format!("Directory {} does not contain a circuit", root.display()).into());
        }

        let create_spinner = style_spinner(
            ProgressBar::new_spinner(),
            &format!("⏳ Checking contract for circuit at {}...", root.display()),
        );

        // run stylus check in contracts directory
        match self.stylus.check(&contracts_root, &rpc_url.unwrap_or("https://sepolia-rollup.arbitrum.io/rpc".to_string())) {
            Ok(result) => {
                create_spinner.finish_with_message(format!(
                    "{} Checked contract for circuit at {}\n",
                    "✅ Success!".green(),
                    root.display()
                ));
                println!("{}", result);
            }
            Err(e) => {
                create_spinner.finish_with_message(format!(
                    "{} Checked contract for circuit at {}\n",
                    "❌ Error!".red(),
                    root.display()
                ));
                return Err(e);
            }
        }

        Ok(())
    }
}
