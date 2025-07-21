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
        } else if !self.system.exists(&contracts_root) {
            return Err(format!(
                "We can't find your contracts at {}. Please run generate first.",
                contracts_root.display()
            )
            .into());
        }

        let spinner = style_spinner(
            ProgressBar::new_spinner(),
            &format!("⏳ Deploying {}...", root.display()),
        );

        let verifier_address = match verifier_address {
            Some(address) => address,
            None => {
                spinner.set_message("Determining default verifier address...");

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

        spinner.set_message("Deploying...");
        match self
            .stylus
            .deploy(&contracts_root, &rpc_url, &private_key, &verifier_address)
        {
            Ok(result) => {
                spinner.finish_with_message(format!(
                    "{} Deployed {}\n",
                    "✅ Success!".green(),
                    root.display()
                ));
                println!("{result}");
            }
            Err(e) => {
                spinner.finish_with_message(format!(
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

    // default values for testing
    const RPC_URL: &str = "https://rpc.sepolia.org";
    const PRIVATE_KEY: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
    const VERIFIER_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
    const ROOT: &str = "circuit";
    const CONTRACTS_ROOT: &str = "circuit/contracts";

    /// Basic test case, user provides all parameters.
    /// We test we properly run verifications and call the deployment with the correct parameters.
    #[tokio::test]
    async fn test_deploy_command() {
        // rpc should not be called as we provide verifier address
        let rpc_mock = MockTRpc::new();

        // validate stylus is installed
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == CARGO_STYLUS_REQUIREMENT)
            .returning(|_| Ok(()));

        // validate we are in a circuit directory
        let mut system_mock = MockTSystem::new();
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(ROOT).join("Nargo.toml")))
            .returning(|_| true);
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(CONTRACTS_ROOT)))
            .returning(|_| true);

        // call stylus deploy
        let mut stylus_mock = MockTStylus::new();
        stylus_mock
            .expect_deploy()
            .with(
                eq(PathBuf::from(CONTRACTS_ROOT)),
                eq(RPC_URL),
                eq(PRIVATE_KEY),
                eq(VERIFIER_ADDRESS),
            )
            .returning(|_, _, _, _| Ok("".to_string()));

        // run deploy command
        let result = DeployCommand {
            system_requirements_checker: Box::new(system_requirements_checker_mock),
            stylus: Box::new(stylus_mock),
            system: Box::new(system_mock),
            rpc: Box::new(rpc_mock),
        }
        .run(
            &AppContext {},
            Some(ROOT.to_string()),
            RPC_URL.to_string(),
            PRIVATE_KEY.to_string(),
            Some(VERIFIER_ADDRESS.to_string()),
            false,
        )
        .await;

        // assert result is ok, rest is handled by mock
        assert!(result.is_ok());
    }

    /// If user does not provide verifier address, we should use the default one.
    /// This test checks that we properly determine the default verifier address based on the rpc provided.
    #[tokio::test]
    async fn test_deploy_default_verifier_address() {
        // validate stylus is installed
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == CARGO_STYLUS_REQUIREMENT)
            .returning(|_| Ok(()));

        // validate we are in a circuit directory
        let mut system_mock = MockTSystem::new();
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(ROOT).join("Nargo.toml")))
            .returning(|_| true);
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(CONTRACTS_ROOT)))
            .returning(|_| true);

        // call rpc get chain id to later determine default verifier address
        let mut rpc_mock = MockTRpc::new();
        rpc_mock
            .expect_get_chain_id()
            .with(eq(RPC_URL))
            .returning(|_| Box::pin(async { Ok(CHAIN_ID_ARBITRUM_SEPOLIA) }));

        // call stylus deploy
        let mut stylus_mock = MockTStylus::new();
        stylus_mock
            .expect_deploy()
            .with(
                eq(PathBuf::from(CONTRACTS_ROOT)),
                eq(RPC_URL),
                eq(PRIVATE_KEY),
                eq(VERIFIER_ADDRESS_ARBITRUM_SEPOLIA),
            )
            .returning(|_, _, _, _| Ok("".to_string()));

        // run deploy command
        let result = DeployCommand {
            system_requirements_checker: Box::new(system_requirements_checker_mock),
            stylus: Box::new(stylus_mock),
            system: Box::new(system_mock),
            rpc: Box::new(rpc_mock),
        }
        .run(
            &AppContext {},
            Some(ROOT.to_string()),
            RPC_URL.to_string(),
            PRIVATE_KEY.to_string(),
            None,
            false,
        )
        .await;

        // assert result is ok, rest is handled by mock
        assert!(result.is_ok());
    }
}
