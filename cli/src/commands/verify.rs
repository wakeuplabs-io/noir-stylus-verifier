use crate::{
    config::constants::DEFAULT_RPC_URL,
    infrastructure::{
        bb::{Bb, TBb},
        progress::create_spinner,
        system::{System, TSystem},
    },
    print_error, print_warning, AppContext, AppError,
};
use alloy::{primitives::Bytes, providers::ProviderBuilder, sol};
use colored::*;
use std::path::PathBuf;

pub(crate) struct VerifyCommand {
    system: Box<dyn TSystem>,
    bb: Box<dyn TBb>,
}

impl Default for VerifyCommand {
    fn default() -> Self {
        Self {
            system: Box::new(System),
            bb: Box::new(Bb::default()),
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
    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        proof: Option<String>,
        public_input: Option<String>,
        vk: Option<String>,
        verifier_address: Option<String>,
        rpc_url: Option<String>,
        zk: bool,
    ) -> Result<(), AppError> {
        // find package root
        let root = self.system.current_dir();

        // defaults to target folder
        let proof = PathBuf::from(proof.unwrap_or_else(|| {
            root.join("target")
                .join("proof")
                .to_string_lossy()
                .to_string()
        }));
        let public_input = PathBuf::from(public_input.unwrap_or_else(|| {
            root.join("target")
                .join("public_inputs")
                .to_string_lossy()
                .to_string()
        }));
        let vk = match vk {
            Some(vk) => PathBuf::from(vk),
            None => {
                let vk_path = root.join("contracts").join("assets").join("vk");
                if self.system.exists(&vk_path) {
                    vk_path
                } else if self.system.exists(&root.join("target").join("vk")) {
                    print_warning!("VK not found in contracts/assets, using ./target/vk instead");
                    root.join("target").join("vk")
                } else {
                    return Err(AppError::Other("VK not found"));
                }
            }
        };

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
            None => match self.bb.verify(&root, &proof, &public_input, &vk, zk) {
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
            },
        }

        Ok(())
    }
}
