//! # Check Command
//!
//! The check command validates that generated Stylus verifier contracts are compatible
//! with the Stylus runtime and estimates deployment costs. This command is typically
//! run after generating a verifier contract to ensure it will deploy successfully.

use crate::infrastructure::terminal::print_instructions;
use crate::AppError;
use crate::{
    config::constants::CARGO_STYLUS_REQUIREMENT,
    infrastructure::requirements::{SystemRequirementsChecker, TSystemRequirementsChecker},
    infrastructure::{
        nargo::{Nargo, TNargo},
        progress::create_spinner,
        stylus::{Stylus, TStylus},
        system::{System, TSystem},
    },
    AppContext,
};
use colored::*;

/// Command for checking Stylus contract compatibility and deployment costs.
///
/// This command validates that a generated verifier contract is compatible with
/// the Stylus runtime and provides estimates for deployment gas costs. It's an
/// essential step before deploying contracts to ensure they will work correctly.
pub(crate) struct CheckCommand {
    /// Stylus CLI interface for contract validation
    stylus: Box<dyn TStylus>,
    /// System operations interface
    system: Box<dyn TSystem>,
    /// Nargo CLI interface for Noir operations
    nargo: Box<dyn TNargo>,
    /// System requirements checker
    system_requirements_checker: Box<dyn TSystemRequirementsChecker>,
}

impl Default for CheckCommand {
    fn default() -> Self {
        Self {
            stylus: Box::new(Stylus::default()),
            system: Box::new(System),
            nargo: Box::new(Nargo::default()),
            system_requirements_checker: Box::new(SystemRequirementsChecker::default()),
        }
    }
}

impl CheckCommand {
    /// Executes the check command to validate contract compatibility.
    ///
    /// # Arguments
    ///
    /// * `_ctx` - Application context (currently unused)
    /// * `package` - Optional package name to check. If None, uses current directory
    /// * `rpc_url` - RPC URL for blockchain connection during validation
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the contract passes all checks, or an `AppError` if
    /// validation fails or required files are missing.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Required system dependencies (cargo-stylus) are not installed
    /// - The specified package cannot be found
    /// - No contracts directory exists (indicating `generate` hasn't been run)
    /// - The Stylus validation fails
    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        package: Option<String>,
        rpc_url: String,
    ) -> Result<(), AppError> {
        self.system_requirements_checker
            .check(vec![CARGO_STYLUS_REQUIREMENT])
            .map_err(AppError::MissingDependencies)?;

        // find package root
        let root = match package {
            Some(package) => self
                .nargo
                .find_package_root(&package)
                .map_err(|_| AppError::PackageNotFound)?,
            None => self.system.current_dir(),
        };

        // read package name, double checks root and needed later for nargo and bb
        let package_name = self
            .nargo
            .read_package_name(&root)
            .map_err(|_| AppError::PackageNotFound)?;
        let contracts_root = root.join("contracts");

        // verify we are in a circuit directory.
        if !self.system.exists(&contracts_root) {
            return Err(AppError::ContractsNotFound(contracts_root));
        }

        // all good, we can run the check
        let progress = create_spinner(&format!(
            "⏳ Checking contract for package {} at {}...",
            package_name,
            root.display()
        ));

        // run stylus check in contracts directory
        match self.stylus.check(&contracts_root, &rpc_url) {
            Ok(result) => {
                progress.finish_and_clear();
                println!(
                    "{} Checked contract for package {} at {}\n",
                    "✅ Success!".green(),
                    package_name,
                    root.display()
                );
                println!("{result}");
            }
            Err(e) => {
                progress.finish_and_clear();
                println!(
                    "{} Checked contract for package {} at {}\n",
                    "❌ Error!".red(),
                    package_name,
                    root.display()
                );
                return Err(AppError::StylusError(e.to_string()));
            }
        }

        print_instructions(&["deploy", "prove", "verify"]);

        Ok(())
    }
}
