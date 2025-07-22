use crate::{
    infrastructure::{
        bb::{Bb, TBb},
        nargo::{Nargo, TNargo},
        progress::create_spinner,
        system::{System, TSystem},
    },
    AppContext, AppError,
};
use colored::*;

pub(crate) struct VerifyCommand {
    system: Box<dyn TSystem>,
    nargo: Box<dyn TNargo>,
    bb: Box<dyn TBb>,
}

impl Default for VerifyCommand {
    fn default() -> Self {
        Self {
            system: Box::new(System),
            nargo: Box::new(Nargo::default()),
            bb: Box::new(Bb::default()),
        }
    }
}

impl VerifyCommand {
    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        package: Option<String>,
        proof: Option<String>,
        public_input: Option<String>,
        verifier_address: Option<String>,
        zk: bool,
    ) -> Result<(), AppError> {
        let root = match package {
            Some(package) => self
                .nargo
                .find_package_root(&package)
                .map_err(|_| AppError::PackageNotFound)?,
            None => self.system.current_dir(),
        };
        let package_name = self
            .nargo
            .read_package_name(&root)
            .map_err(|_| AppError::PackageNotFound)?;

        let proof = proof.unwrap_or_else(|| {
            root.join("target")
                .join("proof")
                .to_string_lossy()
                .to_string()
        });
        let public_input = public_input.unwrap_or_else(|| {
            root.join("target")
                .join("public_input")
                .to_string_lossy()
                .to_string()
        });

        // All good, let's verify the proof

        let spinner = create_spinner(&format!("⏳ Verifying proof at {proof}..."));

        match verifier_address {
            Some(address) => {
                // TODO: global vs local
            }
            None => {
                self.bb
                    .verify(&root, &package_name, &proof, &public_input, zk)
                    .map_err(|_| AppError::Other("Failed to verify proof"))?;

                spinner.finish_with_message(format!(
                    "{} Proof verified at {}\n",
                    "✅ Success!".green(),
                    root.join("target").join("proof").display()
                ));
            }
        }

        Ok(())
    }
}
