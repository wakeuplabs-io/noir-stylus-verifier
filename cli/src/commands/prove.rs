use crate::{
    config::requirements::{
        SystemRequirementsChecker, TSystemRequirementsChecker, BB_UP_REQUIREMENT,
        NOIRUP_REQUIREMENT,
    },
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

pub(crate) struct ProveCommand {
    system: Box<dyn TSystem>,
    bb: Box<dyn TBb>,
    nargo: Box<dyn TNargo>,
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
    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        package: Option<String>,
        prover_name: String,
        zk: bool,
    ) -> Result<(), AppError> {
        // verify dependencies
        self.system_requirements_checker
            .check(vec![BB_UP_REQUIREMENT, NOIRUP_REQUIREMENT])
            .map_err(|_| AppError::Other("Failed to verify dependencies"))?;

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

        let spinner = create_spinner(&format!(
            "⏳ Creating proof for {package_name} at:\n\t{}",
            root.display()
        ));

        self.nargo
            .execute(&root, &package_name, &prover_name)
            .map_err(|_| AppError::Other("Failed to execute nargo"))?;
        self.bb
            .prove(&root, &package_name, zk)
            .map_err(|_| AppError::Other("Failed to generate proof"))?;

        spinner.finish_with_message(format!(
            "{} Proof generated at: \n\t{}\n",
            "✅ Success!".green(),
            root.join("target").join("proof").display()
        ));

        print_instructions(&["verify"]);

        Ok(())
    }
}
