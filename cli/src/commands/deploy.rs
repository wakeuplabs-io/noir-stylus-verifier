use crate::{
    config::requirements::{
        SystemRequirementsChecker, TSystemRequirementsChecker, CARGO_STYLUS_REQUIREMENT,
    },
    infrastructure::{console::progress::style_spinner, stylus::Stylus},
    print_warning, AppContext,
};
use colored::*;
use core::panic;
use ethers::providers::{Http, Middleware, Provider};
use indicatif::ProgressBar;
use std::{env, path::PathBuf};

pub(crate) struct DeployCommand {
    system_requirements_checker: SystemRequirementsChecker,
    stylus: Stylus,
}

// cargo run -p nsv deploy --rpc-url http://localhost:8547 --private-key 0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659 --verifier-address 0x0000000000000000000000000000000000000001 

impl DeployCommand {
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
        rpc_url: String,
        private_key: String,
        verifier_address: Option<String>,
        zk_flavor: bool,
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
            &format!("⏳ Deploying {}...", root.display()),
        );

        let verifier_address = match verifier_address {
            Some(address) => address,
            None => {
                // get chain id from rpc url
                let provider = Provider::<Http>::try_from(&rpc_url)?;
                let chain_id = provider.get_chainid().await?.as_u64();

                // select default verifier address from constants.
                let verifier_address = match chain_id {
                    42161 => {
                        if zk_flavor {
                            "0x0000000000000000000000000000000000000000"
                        } else {
                            "0x0000000000000000000000000000000000000000"
                        }
                    } // arbitrum
                    421614 => {
                        if zk_flavor {
                            "0x0000000000000000000000000000000000000000"
                        } else {
                            "0x0000000000000000000000000000000000000000"
                        }
                    } // arbitrum sepolia
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
                println!("{}", result);
            }
            Err(e) => {
                create_spinner.finish_with_message(format!(
                    "{} Failed to deploy {}\n",
                    "❌ Error!".red(),
                    root.display()
                ));
                println!("{}", e);
            }
        }

        Ok(())
    }
}
