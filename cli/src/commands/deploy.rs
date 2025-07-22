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
        nargo::{Nargo, TNargo},
        progress::create_spinner,
        rpc::{Rpc, TRpc},
        stylus::{Stylus, TStylus},
        system::{System, TSystem},
    },
    print_error, print_warning, AppContext, AppError,
};
use colored::*;

pub(crate) struct DeployCommand {
    system_requirements_checker: Box<dyn TSystemRequirementsChecker>,
    stylus: Box<dyn TStylus>,
    system: Box<dyn TSystem>,
    rpc: Box<dyn TRpc>,
    nargo: Box<dyn TNargo>,
}

impl Default for DeployCommand {
    fn default() -> Self {
        Self {
            system_requirements_checker: Box::new(SystemRequirementsChecker::default()),
            stylus: Box::new(Stylus::default()),
            system: Box::new(System),
            rpc: Box::new(Rpc::default()),
            nargo: Box::new(Nargo::default()),
        }
    }
}

impl DeployCommand {
    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        package: Option<String>,
        rpc_url: String,
        private_key: String,
        verifier_address: Option<String>,
        zk_flavor: bool,
    ) -> Result<(), AppError> {
        self.system_requirements_checker
            .check(vec![CARGO_STYLUS_REQUIREMENT])
            .map_err(|_| AppError::MissingDependencies())?;

        let root = match package {
            Some(package) => self
                .nargo
                .find_package_root(&package)
                .map_err(|_| AppError::PackageNotFound)?,
            None => self.system.current_dir(),
        };
        let relative_root = root.strip_prefix(self.system.current_dir()).unwrap();
        let contracts_root = root.join("contracts");

        // verify that contracts were already generated.
        if !self.system.exists(&contracts_root) {
            return Err(AppError::ContractsNotFound(contracts_root));
        }

        let spinner = create_spinner(&format!("⏳ Deploying {}...", relative_root.display()));

        let verifier_address = match verifier_address {
            Some(address) => address,
            None => {
                spinner.set_message("Determining default verifier address...");

                // get chain id from rpc url
                let chain_id = self
                    .rpc
                    .get_chain_id(&rpc_url)
                    .await
                    .map_err(|e| AppError::RpcError(e.to_string()))?;

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
                    _ => panic!("No default verifier address for chain id: {chain_id}"),
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
                    relative_root.display()
                ));
                println!("{result}");
            }
            Err(e) => {
                spinner.finish_with_message(format!(
                    "{} Failed to deploy {}\n",
                    "❌ Error!".red(),
                    relative_root.display()
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
    use crate::infrastructure::{
        nargo::MockTNargo, rpc::MockTRpc, stylus::MockTStylus, system::MockTSystem,
    };
    use mockall::predicate::*;
    use std::path::PathBuf;

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

        let mut nargo_mock = MockTNargo::new();
        nargo_mock
            .expect_find_package_root()
            .with(eq(ROOT))
            .returning(|_| Ok(PathBuf::from(ROOT)));

        // validate contracts were generated
        let mut system_mock = MockTSystem::new();
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
            nargo: Box::new(nargo_mock),
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
        let mut nargo_mock = MockTNargo::new();
        nargo_mock
            .expect_find_package_root()
            .with(eq(ROOT))
            .returning(|_| Ok(PathBuf::from(ROOT)));

        // validate we are in a circuit directory
        let mut system_mock = MockTSystem::new();
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(CONTRACTS_ROOT)))
            .returning(|_| true);

        // call rpc get chain id to later determine default verifier address
        let mut rpc_mock = MockTRpc::new();
        rpc_mock
            .expect_get_chain_id()
            .with(eq(RPC_URL))
            .returning(|_| Ok(CHAIN_ID_ARBITRUM_SEPOLIA));

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
            nargo: Box::new(nargo_mock),
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
