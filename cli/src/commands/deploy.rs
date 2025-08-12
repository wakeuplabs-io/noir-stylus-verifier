//! # Deploy Command
//!
//! The deploy command handles deploying generated Stylus verifier contracts to
//! the blockchain. It automatically detects the appropriate verifier contract
//! address based on the target chain or allows manual specification.

use crate::{
    config::{
        constants::CARGO_STYLUS_REQUIREMENT,
        constants::{
            CHAIN_ID_ARBITRUM, CHAIN_ID_ARBITRUM_SEPOLIA, VERIFIER_ADDRESS_ARBITRUM,
            VERIFIER_ADDRESS_ARBITRUM_SEPOLIA, VERIFIER_ADDRESS_ARBITRUM_SEPOLIA_ZK,
            VERIFIER_ADDRESS_ARBITRUM_ZK,
        },
    },
    infrastructure::{
        nargo::{Nargo, TNargo},
        progress::create_spinner,
        requirements::{SystemRequirementsChecker, TSystemRequirementsChecker},
        rpc::{Rpc, TRpc},
        stylus::{Stylus, TStylus},
        system::{System, TSystem},
        terminal::print_instructions,
    },
    print_error, print_warning, AppContext, AppError,
};
use colored::*;

/// Command for deploying Stylus verifier contracts to the blockchain.
///
/// This command deploys a previously generated verifier contract to a target
/// blockchain network. It handles chain detection, verifier address resolution,
/// and transaction signing automatically.
pub(crate) struct DeployCommand {
    /// System requirements checker
    system_requirements_checker: Box<dyn TSystemRequirementsChecker>,
    /// Stylus CLI interface for deployment
    stylus: Box<dyn TStylus>,
    /// System operations interface
    system: Box<dyn TSystem>,
    /// RPC interface for blockchain interactions
    rpc: Box<dyn TRpc>,
    /// Nargo CLI interface for Noir operations
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
    /// Executes the deploy command to deploy a verifier contract.
    ///
    /// # Arguments
    ///
    /// * `_ctx` - Application context (currently unused)
    /// * `package` - Optional package name to deploy. If None, uses current directory
    /// * `rpc_url` - RPC URL for the target blockchain network
    /// * `private_key` - Private key for signing the deployment transaction
    /// * `verifier_address` - Optional verifier contract address. If None, uses chain defaults
    /// * `zk_flavor` - Whether to use zk-flavored verifier contracts
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if deployment succeeds, or an `AppError` if deployment fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Required system dependencies (cargo-stylus) are not installed
    /// - The specified package cannot be found
    /// - No contracts directory exists (indicating `generate` hasn't been run)
    /// - Chain ID detection fails
    /// - No default verifier address exists for the target chain
    /// - The deployment transaction fails
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
            .map_err(AppError::MissingDependencies)?;

        let root = match package {
            Some(package) => self
                .nargo
                .find_package_root(&package)
                .map_err(|_| AppError::PackageNotFound)?,
            None => self.system.current_dir(),
        };
        let contracts_root = root.join("contracts");

        // verify that contracts were already generated.
        if !self.system.exists(&contracts_root) {
            return Err(AppError::ContractsNotFound(contracts_root));
        }

        let spinner = create_spinner(&format!("⏳ Deploying {}...", contracts_root.display()));

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
                    _ => {
                        return Err(AppError::NoDefaultVerifierAddress);
                    }
                }
                .to_string();

                if verifier_address == "0x0000000000000000000000000000000000000000" {
                    return Err(AppError::NoDefaultVerifierAddress);
                }

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
                    contracts_root.display()
                ));
                println!("{result}");
            }
            Err(e) => {
                spinner.finish_with_message(format!(
                    "{} Failed to deploy {}\n",
                    "❌ Error!".red(),
                    contracts_root.display()
                ));
                print_error!("{e}");
            }
        }

        print_instructions(&["prove", "verify"]);

        Ok(())
    }
}
