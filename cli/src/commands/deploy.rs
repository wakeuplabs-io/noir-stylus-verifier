use crate::{
    config::{
        constants::{
            CHAIN_ID_ARBITRUM, CHAIN_ID_ARBITRUM_SEPOLIA, VERIFIER_ADDRESS_ARBITRUM,
            VERIFIER_ADDRESS_ARBITRUM_SEPOLIA, VERIFIER_ADDRESS_ARBITRUM_SEPOLIA_ZK,
            VERIFIER_ADDRESS_ARBITRUM_ZK,
        },
        constants::CARGO_STYLUS_REQUIREMENT,
    },
    infrastructure::{
        nargo::{Nargo, TNargo},
        progress::create_spinner,
        rpc::{Rpc, TRpc},
        stylus::{Stylus, TStylus},
        system::{System, TSystem},
        terminal::print_instructions,
        requirements::{SystemRequirementsChecker, TSystemRequirementsChecker},
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
            .map_err(|e| AppError::MissingDependencies(e))?;

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
