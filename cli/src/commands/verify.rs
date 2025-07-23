use crate::{
    config::{
        constants::DEFAULT_RPC_URL,
        requirements::{
            SystemRequirementsChecker, TSystemRequirementsChecker, BB_REQUIREMENT,
            BB_UP_REQUIREMENT,
        },
    },
    infrastructure::{
        bb::{Bb, TBb},
        progress::create_spinner,
        system::{System, TSystem},
    },
    print_error, AppContext, AppError,
};
use alloy::{primitives::Bytes, providers::ProviderBuilder, sol};
use colored::*;
use std::path::PathBuf;

pub(crate) struct VerifyCommand {
    system: Box<dyn TSystem>,
    bb: Box<dyn TBb>,
    system_requirements_checker: Box<dyn TSystemRequirementsChecker>,
}

impl Default for VerifyCommand {
    fn default() -> Self {
        Self {
            system: Box::new(System),
            bb: Box::new(Bb::default()),
            system_requirements_checker: Box::new(SystemRequirementsChecker::default()),
        }
    }
}

sol! {
   #[sol(rpc)]
   contract Verifier {
        function verify(bytes proof, bytes public_inputs) public view returns (bool);
   }
}

impl VerifyCommand {
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        proof: String,
        public_input: String,
        vk: String,
        verifier_address: Option<String>,
        rpc_url: Option<String>,
        zk: bool,
    ) -> Result<(), AppError> {
        // verify dependencies
        self.system_requirements_checker
            .check(vec![BB_UP_REQUIREMENT])
            .map_err(|_| AppError::Other("Failed to verify dependencies"))?;

        let root = self.system.current_dir();
        let proof = PathBuf::from(proof);
        let public_input = PathBuf::from(public_input);
        let vk = PathBuf::from(vk);

        // defaults to target folder
        if !self.system.exists(&proof) {
            return Err(AppError::Other("Proof not found"));
        }
        if !self.system.exists(&public_input) {
            return Err(AppError::Other("Public input not found"));
        }
        if !self.system.exists(&vk) {
            return Err(AppError::Other("VK not found"));
        }

        // All good, let's verify the proof

        let spinner = create_spinner(&format!("⏳ Verifying proof at {}...", proof.display()));

        match verifier_address {
            Some(address) => {
                // call the verifier contract with the proof and public inputs
                let provider = ProviderBuilder::new().on_http(
                    rpc_url
                        .unwrap_or(DEFAULT_RPC_URL.to_string())
                        .parse()
                        .unwrap(),
                );

                let proof_bytes: Bytes = self.system.read_file(&proof).into();
                let public_input_bytes: Bytes = self.system.read_file(&public_input).into();
                let result = Verifier::new(address.parse().unwrap(), provider)
                    .verify(proof_bytes, public_input_bytes)
                    .call()
                    .await
                    .map_err(|e| AppError::RpcError(e.to_string()))?;

                if result._0 {
                    spinner.finish_with_message(format!(
                        "{} Proof verified onchain\n",
                        "✅ Success!".green(),
                    ));
                } else {
                    spinner.finish_with_message(format!(
                        "{} Proof verification failed onchain\n",
                        "❌ Error!".red(),
                    ));
                }
            }
            None => {
                self.bb
                    .setup(BB_REQUIREMENT.required_version)
                    .map_err(|_| AppError::Other("Failed to setup bb"))?;

                match self.bb.verify(&root, &proof, &public_input, &vk, zk) {
                    Ok(_) => {
                        spinner.finish_with_message(format!(
                            "{} Proof verified at {}\n",
                            "✅ Success!".green(),
                            root.join("target").join("proof").display()
                        ));
                    }
                    Err(e) => {
                        spinner.finish_with_message(format!(
                            "{} Failed to verify proof\n",
                            "❌ Error!".red()
                        ));
                        print_error!("{e}");
                    }
                }
            }
        }

        Ok(())
    }
}
