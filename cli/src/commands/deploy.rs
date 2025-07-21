use crate::{
    config::{
        constants::{
            CHAIN_ID_ARBITRUM, CHAIN_ID_ARBITRUM_SEPOLIA, VERIFIER_ADDRESS_ARBITRUM,
            VERIFIER_ADDRESS_ARBITRUM_SEPOLIA, VERIFIER_ADDRESS_ARBITRUM_SEPOLIA_ZK,
            VERIFIER_ADDRESS_ARBITRUM_ZK,
        },
        requirements::{
            SystemRequirementsChecker, TSystemRequirementsChecker, CARGO_STYLUS_REQUIREMENT,
        },
    },
    infrastructure::{
        console::progress::style_spinner,
        rpc::{Rpc, TRpc},
        stylus::{Stylus, TStylus},
        system::{System, TSystem},
    },
    print_error, print_warning, AppContext,
};
use colored::*;
use ethers::providers::{Http, Middleware, Provider};
use indicatif::ProgressBar;
use std::{env, path::PathBuf};

pub(crate) struct DeployCommand {
    system_requirements_checker: Box<dyn TSystemRequirementsChecker>,
    stylus: Box<dyn TStylus>,
    system: Box<dyn TSystem>,
    rpc: Box<dyn TRpc>,
}

impl Default for DeployCommand {
    fn default() -> Self {
        Self {
            system_requirements_checker: Box::new(SystemRequirementsChecker::new()),
            stylus: Box::new(Stylus::new()),
            system: Box::new(System::new()),
            rpc: Box::new(Rpc::new()),
        }
    }
}

impl DeployCommand {
    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        circuit: Option<String>,
        rpc_url: String,
        private_key: String,
        verifier_address: Option<String>,
        zk_flavor: bool,
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
            return Err(format!("Directory {} does not contain a circuit", root.display()).into());
        }

        let create_spinner = style_spinner(
            ProgressBar::new_spinner(),
            &format!("⏳ Deploying {}...", root.display()),
        );

        let verifier_address = match verifier_address {
            Some(address) => address,
            None => {
                // get chain id from rpc url
                let chain_id = self.rpc.get_chain_id(&rpc_url).await?;

                // select default verifier address from constants.
                let verifier_address = match chain_id {
                    CHAIN_ID_ARBITRUM => {
                        if zk_flavor {
                            VERIFIER_ADDRESS_ARBITRUM_ZK
                        } else {
                            VERIFIER_ADDRESS_ARBITRUM
                        }
                    }
                    CHAIN_ID_ARBITRUM_SEPOLIA => {
                        if zk_flavor {
                            VERIFIER_ADDRESS_ARBITRUM_SEPOLIA_ZK
                        } else {
                            VERIFIER_ADDRESS_ARBITRUM_SEPOLIA
                        }
                    }
                    _ => panic!("No default verifier address for chain id: {}", chain_id),
                }
                .to_string();

                print_warning!(
                    "Using default verifier address for chain {}: {}",
                    chain_id,
                    verifier_address
                );

                verifier_address
            }
        };

        match self
            .stylus
            .deploy(&contracts_root, &rpc_url, &private_key, &verifier_address)
        {
            Ok(result) => {
                create_spinner.finish_with_message(format!(
                    "{} Deployed {}\n",
                    "✅ Success!".green(),
                    root.display()
                ));
                println!("{result}");
            }
            Err(e) => {
                create_spinner.finish_with_message(format!(
                    "{} Failed to deploy {}\n",
                    "❌ Error!".red(),
                    root.display()
                ));
                print_error!("{e}");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::requirements::MockTSystemRequirementsChecker;
    use crate::infrastructure::{rpc::MockTRpc, stylus::MockTStylus, system::MockTSystem};
    use mockall::{predicate::*, *};

    #[tokio::test]
    async fn test_deploy_command() {
        let rpc_mock = MockTRpc::new();
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let mut stylus_mock = MockTStylus::new();
        let mut system_mock = MockTSystem::new();

        let rpc_url = "https://rpc.sepolia.org";
        let private_key = "0x0000000000000000000000000000000000000000000000000000000000000000";
        let verifier_address = "0x0000000000000000000000000000000000000000";
        let root = PathBuf::from("circuit");
        let contracts_root = root.join("contracts");

        // validate stylus is installed
        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == CARGO_STYLUS_REQUIREMENT)
            .returning(|_| Ok(()));

        // validate we are in a circuit directory
        system_mock
            .expect_exists()
            .with(eq(root.join("Nargo.toml")))
            .returning(|_| true);

        // call stylus deploy
        stylus_mock
            .expect_deploy()
            .with(
                eq(contracts_root),
                eq(rpc_url),
                eq(private_key),
                eq(verifier_address),
            )
            .returning(|_, _, _, _| Ok("".to_string()));

        let command = DeployCommand {
            system_requirements_checker: Box::new(system_requirements_checker_mock),
            stylus: Box::new(stylus_mock),
            system: Box::new(system_mock),
            rpc: Box::new(rpc_mock),
        };
        let ctx = AppContext {};

        let result = command
            .run(
                &ctx,
                Some(root.to_str().unwrap().to_string()),
                rpc_url.to_string(),
                private_key.to_string(),
                Some(verifier_address.to_string()),
                false,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_deploy_default_verifier_address() {
        let mut rpc_mock = MockTRpc::new();
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let mut stylus_mock = MockTStylus::new();
        let mut system_mock = MockTSystem::new();

        let rpc_url = "https://rpc.sepolia.org";
        let private_key = "0x0000000000000000000000000000000000000000000000000000000000000000";
        let root = PathBuf::from("circuit");
        let contracts_root = root.join("contracts");

        // validate stylus is installed
        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == CARGO_STYLUS_REQUIREMENT)
            .returning(|_| Ok(()));

        // validate we are in a circuit directory
        system_mock
            .expect_exists()
            .with(eq(root.join("Nargo.toml")))
            .returning(|_| true);

        // call rpc get chain id to later determine default verifier address
        rpc_mock
            .expect_get_chain_id()
            .with(eq(rpc_url))
            .returning(|_| Box::pin(async { Ok(CHAIN_ID_ARBITRUM_SEPOLIA) }));

        // call stylus deploy
        stylus_mock
            .expect_deploy()
            .with(
                eq(contracts_root),
                eq(rpc_url),
                eq(private_key),
                eq(VERIFIER_ADDRESS_ARBITRUM_SEPOLIA),
            )
            .returning(|_, _, _, _| Ok("".to_string()));

        let command = DeployCommand {
            system_requirements_checker: Box::new(system_requirements_checker_mock),
            stylus: Box::new(stylus_mock),
            system: Box::new(system_mock),
            rpc: Box::new(rpc_mock),
        };
        let ctx = AppContext {};

        let result = command
            .run(
                &ctx,
                Some(root.to_str().unwrap().to_string()),
                rpc_url.to_string(),
                private_key.to_string(),
                None,
                false,
            )
            .await;
        assert!(result.is_ok());
    }
}
