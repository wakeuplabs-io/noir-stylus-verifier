use crate::{
    config::{
        constants::DEFAULT_RPC_URL,
        requirements::{
            SystemRequirementsChecker, TSystemRequirementsChecker, CARGO_STYLUS_REQUIREMENT,
        },
    },
    infrastructure::{
        console::progress::style_spinner,
        stylus::{Stylus, TStylus},
        system::{System, TSystem},
    },
    print_warning, AppContext,
};
use colored::*;
use indicatif::ProgressBar;
use std::{env, path::PathBuf};

pub(crate) struct CheckCommand {
    system_requirements_checker: Box<dyn TSystemRequirementsChecker>,
    stylus: Box<dyn TStylus>,
    system: Box<dyn TSystem>,
}

impl Default for CheckCommand {
    fn default() -> Self {
        Self {
            system_requirements_checker: Box::new(SystemRequirementsChecker::new()),
            stylus: Box::new(Stylus::new()),
            system: Box::new(System::new()),
        }
    }
}

impl CheckCommand {
    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        circuit: Option<String>,
        rpc_url: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.system_requirements_checker
            .check(vec![CARGO_STYLUS_REQUIREMENT])?;

        let root = if let Some(circuit) = circuit {
            PathBuf::from(circuit)
        } else {
            env::current_dir()?
        };
        let contracts_root = root.join("contracts");

        // verify we are in a circuit directory.
        if !self.system.exists(&root.join("Nargo.toml")) {
            return Err(format!("We can't find your circuit at {}. Please run this command from the root of your circuit.", root.display()).into());
        }

        // get rpc url from command line or use default
        let rpc_url = match rpc_url {
            Some(url) => url,
            None => {
                print_warning!(
                    "No RPC URL provided, using default RPC URL: {}",
                    DEFAULT_RPC_URL
                );

                DEFAULT_RPC_URL.to_string()
            }
        };

        let progress = style_spinner(
            ProgressBar::new_spinner(),
            &format!("⏳ Checking contract for circuit at {}...", root.display()),
        );

        // run stylus check in contracts directory
        match self.stylus.check(&contracts_root, &rpc_url) {
            Ok(result) => {
                progress.finish_with_message(format!(
                    "{} Checked contract for circuit at {}\n",
                    "✅ Success!".green(),
                    root.display()
                ));
                println!("{result}");
            }
            Err(e) => {
                progress.finish_with_message(format!(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::requirements::MockTSystemRequirementsChecker;
    use crate::infrastructure::{stylus::MockTStylus, system::MockTSystem};
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_check_command() {
        let mut mock_stylus = MockTStylus::new();
        let mut mock_system = MockTSystem::new();
        let mut mock_system_requirements_checker = MockTSystemRequirementsChecker::new();

        let circuit_root = PathBuf::from("circuit");
        let contracts_root = circuit_root.join("contracts");
        let rpc_url = "https://sepolia-rollup.arbitrum.io/rpc";

        // should check we have stylus installed
        mock_system_requirements_checker
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == CARGO_STYLUS_REQUIREMENT)
            .returning(|_| Ok(()));

        // should check we're at the circuit root
        mock_system
            .expect_exists()
            .with(eq(circuit_root.join("Nargo.toml")))
            .returning(|_| true);

        // should run stylus check
        mock_stylus
            .expect_check()
            .with(eq(contracts_root), eq(rpc_url))
            .returning(|_, _| Ok("✅ Success!".to_string()));

        let check_command = CheckCommand {
            system_requirements_checker: Box::new(mock_system_requirements_checker),
            stylus: Box::new(mock_stylus),
            system: Box::new(mock_system),
        };
        let ctx = AppContext {};

        let result = check_command
            .run(
                &ctx,
                Some(circuit_root.to_str().unwrap().to_string()),
                Some(rpc_url.to_string()),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn check_uses_sepolia_as_default_rpc_url() {
        let mut mock_stylus = MockTStylus::new();
        let mut mock_system = MockTSystem::new();
        let mut mock_system_requirements_checker = MockTSystemRequirementsChecker::new();

        let circuit_root = PathBuf::from("circuit");
        let contracts_root = circuit_root.join("contracts");

        // should check we have stylus installed
        mock_system_requirements_checker
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == CARGO_STYLUS_REQUIREMENT)
            .returning(|_| Ok(()));

        // should check we're at the circuit root
        mock_system
            .expect_exists()
            .with(eq(circuit_root.join("Nargo.toml")))
            .returning(|_| true);

        // should run stylus check
        mock_stylus
            .expect_check()
            .with(
                eq(contracts_root),
                eq("https://sepolia-rollup.arbitrum.io/rpc"),
            )
            .returning(|_, _| Ok("✅ Success!".to_string()));

        let check_command = CheckCommand {
            system_requirements_checker: Box::new(mock_system_requirements_checker),
            stylus: Box::new(mock_stylus),
            system: Box::new(mock_system),
        };
        let ctx = AppContext {};

        let result = check_command
            .run(&ctx, Some(circuit_root.to_str().unwrap().to_string()), None)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn check_project_root_fails() {
        let mut mock_stylus = MockTStylus::new();
        let mut mock_system = MockTSystem::new();
        let mut mock_system_requirements_checker = MockTSystemRequirementsChecker::new();

        let circuit_root = PathBuf::from("circuit");

        // should check we have stylus installed
        mock_system_requirements_checker
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == CARGO_STYLUS_REQUIREMENT)
            .returning(|_| Ok(()));

        // should check we're at the circuit root
        mock_system
            .expect_exists()
            .with(eq(circuit_root.join("Nargo.toml")))
            .returning(|_| false);

        // should not run stylus check if we can't find the contracts directory
        mock_stylus.expect_check().never();

        let check_command = CheckCommand {
            system_requirements_checker: Box::new(mock_system_requirements_checker),
            stylus: Box::new(mock_stylus),
            system: Box::new(mock_system),
        };
        let ctx = AppContext {};

        let result = check_command
            .run(&ctx, Some(circuit_root.to_str().unwrap().to_string()), None)
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "We can't find your circuit at circuit. Please run this command from the root of your circuit.");
    }
}
