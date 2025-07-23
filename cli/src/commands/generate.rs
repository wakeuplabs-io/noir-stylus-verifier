use crate::{
    config::requirements::{
        SystemRequirementsChecker, TSystemRequirementsChecker, BB_REQUIREMENT, BB_UP_REQUIREMENT,
        NOIRUP_REQUIREMENT, NOIR_REQUIREMENT,
    },
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

pub(crate) struct GenerateCommand {
    system: Box<dyn TSystem>,
    system_requirements_checker: Box<dyn TSystemRequirementsChecker>,
    verifier_generator: Box<dyn TCodegen>,
    nargo: Box<dyn TNargo>,
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
    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        package: Option<String>,
    ) -> Result<(), AppError> {
        // check system requirements for this command
        self.system_requirements_checker
            .check(vec![BB_UP_REQUIREMENT, NOIRUP_REQUIREMENT])
            .map_err(|_| AppError::MissingDependencies())?;

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

        // all good, we can start generating the verifier contract
        let spinner = create_spinner(&format!(
            "⏳ Generating verifier contract for {}...",
            contracts_root.display()
        ));

        // create contracts directory if it doesn't exist
        self.system.ensure_dir(&contracts_root);

        // set noir version
        spinner.set_message(format!(
            "Setting noir version to {}...",
            NOIR_REQUIREMENT.required_version
        ));
        self.nargo
            .setup(NOIR_REQUIREMENT.required_version)
            .map_err(|_| AppError::Other("Failed to setup noir"))?;

        // compile circuit
        spinner.set_message("Compiling circuit...");
        let bytecode_path = contracts_root.join("assets").join("bytecode.json");
        self.nargo
            .compile(&root, &package_name, &bytecode_path)
            .map_err(|_| AppError::CompileError)?;

        // set bb version and write vk
        spinner.set_message(format!(
            "Setting bb version to {}...",
            BB_REQUIREMENT.required_version
        ));
        self.bb
            .setup(BB_REQUIREMENT.required_version)
            .map_err(|_| AppError::Other("Failed to setup bb"))?;

        // write vk
        spinner.set_message("Writing vk...");
        let vk_path = contracts_root.join("assets").join("vk");
        self.bb
            .write_vk(&root, &package_name, &vk_path)
            .map_err(|_| AppError::Other("Failed to write vk"))?;

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

        spinner.finish_with_message(format!(
            "{} Generated at {}\n",
            "✅ Success!".green(),
            contracts_root.display()
        ));

        print_instructions(&["check", "deploy"]);

        Ok(())
    }
}
