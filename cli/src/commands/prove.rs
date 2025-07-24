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

            witness = root.join("target").join("witness.gz");
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
