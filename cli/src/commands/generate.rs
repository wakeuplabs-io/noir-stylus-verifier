//! # Generate Command
//!
//! The generate command creates Stylus verifier contracts from Noir circuits.
//! It handles circuit compilation, verification key generation, and Rust contract
//! code generation to produce deployable Stylus contracts.

use std::path::Path;

use crate::{
    config::constants::{BB_REQUIREMENT, NARGO_REQUIREMENT},
    infrastructure::requirements::{SystemRequirementsChecker, TSystemRequirementsChecker},
    infrastructure::{
        bb::{Bb, TBb},
        codegen::{Codegen, TCodegen},
        nargo::{Nargo, TNargo},
        progress::create_spinner,
        system::{System, TSystem},
        terminal::print_instructions,
    },
    AppContext, AppError,
};
use colored::*;

/// Command for generating Stylus verifier contracts from Noir circuits.
///
/// This command orchestrates the complete process of converting a Noir circuit
/// into a deployable Stylus verifier contract, including compilation, key generation,
/// and Rust code generation.
pub(crate) struct GenerateCommand {
    /// System operations interface
    system: Box<dyn TSystem>,
    /// System requirements checker
    system_requirements_checker: Box<dyn TSystemRequirementsChecker>,
    /// Code generation interface for creating verifier contracts
    verifier_generator: Box<dyn TCodegen>,
    /// Nargo CLI interface for Noir circuit operations
    nargo: Box<dyn TNargo>,
    /// Barretenberg interface for cryptographic operations
    bb: Box<dyn TBb>,
}

impl Default for GenerateCommand {
    fn default() -> Self {
        Self {
            system: Box::new(System),
            system_requirements_checker: Box::new(SystemRequirementsChecker::default()),
            verifier_generator: Box::new(Codegen::default()),
            nargo: Box::new(Nargo::default()),
            bb: Box::new(Bb::default()),
        }
    }
}

impl GenerateCommand {
    /// Executes the generate command to create a verifier contract.
    ///
    /// # Arguments
    ///
    /// * `_ctx` - Application context (currently unused)
    /// * `package` - Optional package name to generate for. If None, uses current directory
    /// * `bytecode_path` - Optional path to pre-compiled bytecode. If None, compiles the circuit
    /// * `vk_path` - Optional path to verification key. If None, generates the key
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if generation succeeds, or an `AppError` if any step fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Required system dependencies (bb, nargo) are not installed
    /// - The specified package cannot be found
    /// - Provided bytecode or verification key files don't exist
    /// - Circuit compilation fails
    /// - Verification key generation fails
    /// - Contract code generation fails
    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        package: Option<String>,
        bytecode_path: Option<String>,
        vk_path: Option<String>,
    ) -> Result<(), AppError> {
        // check system requirements for this command
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
        let contracts_root = root.join("contracts");

        // read package name, double checks root and needed later for nargo and bb
        let package_name = self
            .nargo
            .read_package_name(&root)
            .map_err(|_| AppError::PackageNotFound)?;

        // verify that the provided bytecode and vk files exist
        if let Some(bytecode_path) = &bytecode_path {
            let bytecode_full_path = root.join(bytecode_path);
            if !self.system.exists(&bytecode_full_path) {
                return Err(AppError::FileNotFound(bytecode_full_path));
            }
        }

        if let Some(vk_path) = &vk_path {
            let vk_full_path = root.join(vk_path);
            if !self.system.exists(&vk_full_path) {
                return Err(AppError::FileNotFound(vk_full_path));
            }
        }

        // all good, we can start generating the verifier contract
        let spinner = create_spinner(&format!(
            "⏳ Generating verifier contract for {}...",
            contracts_root.display()
        ));

        // create contracts directory if it doesn't exist
        self.system.ensure_dir(&contracts_root);

        let target_bytecode_path = contracts_root.join("assets").join("bytecode.json");
        match bytecode_path {
            Some(bytecode_path) => {
                // Using provided bytecode, just copy it to the target path
                spinner.set_message("Using provided bytecode...");
                self.system
                    .copy_file(Path::new(&bytecode_path), &target_bytecode_path);
            }
            None => {
                // Compile circuit
                spinner.set_message("Compiling circuit...");
                self.nargo
                    .compile(&root, &package_name, &target_bytecode_path)
                    .map_err(|_| AppError::CompileError)?;
            }
        }

        let target_vk_path = contracts_root.join("assets").join("vk");
        match vk_path {
            Some(vk_path) => {
                // Using provided vk, just copy it to the target path
                spinner.set_message("Using provided vk...");
                self.system.copy_file(Path::new(&vk_path), &target_vk_path);
            }
            None => {
                // We need to generate the vk
                spinner.set_message("Writing vk...");
                self.bb
                    .write_vk(&root, &target_bytecode_path, &target_vk_path)
                    .map_err(|_| AppError::Other("Failed to write vk"))?;
            }
        }

        // generate verifier contract
        let project_files = self
            .verifier_generator
            .generate_verifier_contract(&package_name)
            .map_err(|_| AppError::GenerateError)?;

        // write project files
        for file in project_files {
            spinner.set_message(format!("Writing {}", file.path));
            self.system
                .write_file(&contracts_root.join(file.path), file.content);
        }

        spinner.finish_and_clear();
        println!(
            "{} Generated at {}\n",
            "✅ Success!".green(),
            contracts_root.display()
        );
        print_instructions(&["check", "deploy", "prove", "verify"]);

        Ok(())
    }
}
