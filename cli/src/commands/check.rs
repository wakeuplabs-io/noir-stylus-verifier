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

pub(crate) struct CheckCommand {
    stylus: Box<dyn TStylus>,
    system: Box<dyn TSystem>,
    nargo: Box<dyn TNargo>,
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
    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        package: Option<String>,
        rpc_url: String,
    ) -> Result<(), AppError> {
        self.system_requirements_checker
            .check(vec![CARGO_STYLUS_REQUIREMENT])
            .map_err(|e| AppError::MissingDependencies(e))?;

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
                println!("{} Checked contract for package {} at {}\n", "✅ Success!".green(), package_name, root.display());
                println!("{result}");
            }
            Err(e) => {
                progress.finish_and_clear();
                println!("{} Checked contract for package {} at {}\n", "❌ Error!".red(), package_name, root.display());
                return Err(AppError::StylusError(e.to_string()));
            }
        }

        print_instructions(&["deploy", "prove", "verify"]);

        Ok(())
    }
}
