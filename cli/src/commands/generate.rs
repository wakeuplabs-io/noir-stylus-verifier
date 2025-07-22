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
        let relative_root = root.strip_prefix(self.system.current_dir()).unwrap();

        // read package name, double checks root and needed later for nargo and bb
        let package_name = self
            .nargo
            .read_package_name(&root)
            .map_err(|_| AppError::PackageNotFound)?;

        // all good, we can start generating the verifier contract
        let spinner = create_spinner(&format!(
            "⏳ Generating verifier contract for {}...",
            relative_root.display()
        ));

        // create contracts directory
        let contracts_root = root.join("contracts");
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
        self.nargo
            .compile(&root)
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
        let circuit_json_path = root.join("target").join(format!("{package_name}.json"));

        let project_files = self
            .verifier_generator
            .generate_verifier_contract(&circuit_json_path)
            .map_err(|_| AppError::GenerateError)?;

        // write project files
        for file in project_files {
            spinner.set_message(format!("Writing {}", file.path));
            self.system
                .write_file(&contracts_root.join(file.path), file.content);
        }

        spinner.finish_with_message(format!(
            "{} Generated at ./{}\n",
            "✅ Success!".green(),
            relative_root.display()
        ));

        // print instructions ========================================

        println!(
            "\n {title}\n\n  - {bin} {check_cmd}: Runs `stylus check` on the generated contract.\n  - {bin} {deploy_cmd}: Deploys the verifier to the blockchain.\n",
            title = "What's Next?".bright_white().bold(),
            bin = env!("CARGO_BIN_NAME").blue(),
            check_cmd = "check".blue(),
            deploy_cmd = "deploy".blue() 
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::requirements::MockTSystemRequirementsChecker;
    use crate::infrastructure::codegen::ProjectFile;
    use crate::infrastructure::{
        bb::MockTBb, codegen::MockTCodegen, nargo::MockTNargo, system::MockTSystem,
    };
    use mockall::predicate::*;
    use std::path::PathBuf;

    const ROOT: &str = "circuit";
    const PACKAGE_NAME: &str = "hello_world";
    const VK_PATH: &str = "circuit/contracts/assets/vk";
    const CONTRACTS_ROOT: &str = "circuit/contracts";

    /// Basic test case, all parameters are given and correct.
    /// We check calls are as expected.
    #[tokio::test]
    async fn test_generate_command() {
        // emulate verifier generator output, and double check we write the files correctly.
        let mock_files: Vec<ProjectFile> = vec![ProjectFile {
            path: "contracts/demo.txt".to_string(),
            content: "demo".to_string(),
        }];

        // ensure we check both bb and noirup are installed
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| {
                reqs.len() == 2 && reqs[0] == BB_UP_REQUIREMENT && reqs[1] == NOIRUP_REQUIREMENT
            })
            .returning(|_| Ok(()));

        // ensure we're at root and can read the package name. Then create the contracts directory and write the verifier outputs.
        let mut system_mock = MockTSystem::new();
        system_mock
            .expect_ensure_dir()
            .with(eq(PathBuf::from(CONTRACTS_ROOT)))
            .returning(|_| ());
        system_mock
            .expect_write_file()
            .with(
                eq(PathBuf::from(CONTRACTS_ROOT)
                    .join(&mock_files[0].path)
                    .clone()),
                eq(mock_files[0].content.clone()),
            )
            .returning(|_, _| ());

        // We need specific version of nargo to ensure compatibility.
        let mut nargo_mock = MockTNargo::new();
        nargo_mock
            .expect_find_package_root()
            .with(eq(PACKAGE_NAME.to_string()))
            .returning(|_| Ok(PathBuf::from(ROOT)));
        nargo_mock
            .expect_setup()
            .with(eq(NOIR_REQUIREMENT.required_version))
            .returning(|_| Ok(()));
        nargo_mock
            .expect_read_package_name()
            .with(eq(PathBuf::from(ROOT)))
            .returning(|_| Ok(PACKAGE_NAME.to_string()));
        nargo_mock.expect_compile().returning(|_| Ok(()));

        // Same for bb
        let mut bb_mock = MockTBb::new();
        bb_mock
            .expect_setup()
            .with(eq(BB_REQUIREMENT.required_version))
            .returning(|_| Ok(()));
        bb_mock
            .expect_write_vk()
            .with(
                eq(PathBuf::from(ROOT)),
                eq(PACKAGE_NAME.to_string()),
                eq(PathBuf::from(VK_PATH)),
            )
            .returning(|_, _, _| Ok(()));

        // ensure we generate the verifier contract
        let mut codegen_mock = MockTCodegen::new();
        let circuit_json_path = PathBuf::from(ROOT)
            .join("target")
            .join(format!("{PACKAGE_NAME}.json"));
        codegen_mock
            .expect_generate_verifier_contract()
            .with(eq(circuit_json_path))
            .returning(move |_| Ok(mock_files.clone()));

        // run the command
        let result = GenerateCommand {
            system: Box::new(system_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
            verifier_generator: Box::new(codegen_mock),
            nargo: Box::new(nargo_mock),
            bb: Box::new(bb_mock),
        }
        .run(&AppContext {}, Some(PACKAGE_NAME.to_string()))
        .await;

        assert!(result.is_ok());
    }
}
