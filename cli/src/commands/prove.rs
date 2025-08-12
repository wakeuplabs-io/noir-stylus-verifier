//! # Prove Command
//!
//! The prove command generates cryptographic proofs for Noir circuits. It can either
//! execute the circuit to generate a witness and then prove it, or use pre-existing
//! witness and bytecode files for proof generation.

use std::path::PathBuf;

use crate::{
    config::constants::{BB_REQUIREMENT, NARGO_REQUIREMENT},
    infrastructure::requirements::{SystemRequirementsChecker, TSystemRequirementsChecker},
    infrastructure::{
        bb::{Bb, TBb},
        nargo::{Nargo, TNargo},
        progress::create_spinner,
        system::{System, TSystem},
        terminal::print_instructions,
    },
    AppContext, AppError,
};
use colored::*;

/// Command for generating cryptographic proofs from Noir circuits.
///
/// This command handles the complete proof generation process, from circuit
/// execution to witness generation and final proof creation using Barretenberg.
pub(crate) struct ProveCommand {
    /// System operations interface
    system: Box<dyn TSystem>,
    /// Barretenberg interface for proof generation
    bb: Box<dyn TBb>,
    /// Nargo CLI interface for circuit execution
    nargo: Box<dyn TNargo>,
    /// System requirements checker
    system_requirements_checker: Box<dyn TSystemRequirementsChecker>,
}

impl Default for ProveCommand {
    fn default() -> Self {
        Self {
            system: Box::new(System),
            bb: Box::new(Bb::default()),
            nargo: Box::new(Nargo::default()),
            system_requirements_checker: Box::new(SystemRequirementsChecker::default()),
        }
    }
}

impl ProveCommand {
    /// Executes the prove command to generate a cryptographic proof.
    ///
    /// # Arguments
    ///
    /// * `_ctx` - Application context (currently unused)
    /// * `package` - Optional package name to prove for. If None, uses current directory
    /// * `prover_path` - Path to the prover configuration file (typically Prover.toml)
    /// * `output_path` - Directory where proof and public inputs will be written
    /// * `witness_path` - Optional path to pre-existing witness. If None, executes circuit
    /// * `bytecode_path` - Optional path to pre-existing bytecode. If None, uses compiled bytecode
    /// * `zk` - Whether to generate a zero-knowledge proof
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if proof generation succeeds, or an `AppError` if any step fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Required system dependencies (bb, nargo) are not installed
    /// - The specified package cannot be found
    /// - Provided witness or bytecode files don't exist
    /// - Circuit execution fails
    /// - Proof generation fails
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        package: Option<String>,
        prover_path: String,
        output_path: String,
        witness_path: Option<String>,
        bytecode_path: Option<String>,
        zk: bool,
    ) -> Result<(), AppError> {
        // verify dependencies
        self.system_requirements_checker
            .check(vec![BB_REQUIREMENT, NARGO_REQUIREMENT])
            .map_err(AppError::MissingDependencies)?;

        // find package root
        let root = match package {
            Some(package) => self
                .nargo
                .find_package_root(&package)
                .map_err(|_| AppError::PackageNotFound)?,
            None => self.system.current_dir(),
        };

        // Read package name, double checks root and needed later for nargo and bb
        let package_name = self
            .nargo
            .read_package_name(&root)
            .map_err(|_| AppError::PackageNotFound)?;

        // All good, let's generate the proof
        let spinner = create_spinner("⏳ Creating proof...");

        let witness: PathBuf;
        let bytecode: PathBuf;
        if witness_path.is_some() && bytecode_path.is_some() {
            // Using provided witness and bytecode
            spinner.set_message("Using provided witness...");

            // check they exist
            if !self
                .system
                .exists(&root.join(witness_path.as_ref().unwrap()))
            {
                return Err(AppError::Other("Witness file does not exist"));
            }
            if !self
                .system
                .exists(&root.join(bytecode_path.as_ref().unwrap()))
            {
                return Err(AppError::Other("Bytecode file does not exist"));
            }

            witness = root.join(witness_path.unwrap());
            bytecode = root.join(bytecode_path.unwrap());
        } else {
            // execute circuit to generate the witness
            spinner.set_message("Generating witness...");
            self.nargo
                .execute(&root, &package_name, &prover_path)
                .map_err(|_| AppError::Other("Failed to execute nargo"))?;

            witness = root.join("target").join(format!("{package_name}.gz"));
            bytecode = root.join("target").join(format!("{package_name}.json"));
        }

        // generate proof
        spinner.set_message("Generating proof...");
        self.bb
            .prove(&root, &bytecode, &witness, &root.join(&output_path), zk)
            .map_err(|_| AppError::Other("Failed to generate proof"))?;

        spinner.finish_and_clear();
        println!(
            "{} Proof generated at: \n\t{}\n",
            "✅ Success!".green(),
            root.join(output_path).display()
        );
        print_instructions(&["verify"]);

        Ok(())
    }
}
