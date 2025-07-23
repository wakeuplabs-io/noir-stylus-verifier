use crate::AppError;
use crate::{
    config::{
        constants::DEFAULT_RPC_URL,
        requirements::{
            SystemRequirementsChecker, TSystemRequirementsChecker, CARGO_STYLUS_REQUIREMENT,
        },
    },
    infrastructure::{
        nargo::{Nargo, TNargo},
        progress::create_spinner,
        stylus::{Stylus, TStylus},
        system::{System, TSystem},
    },
    print_warning, AppContext,
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
        rpc_url: Option<String>,
    ) -> Result<(), AppError> {
        self.system_requirements_checker
            .check(vec![CARGO_STYLUS_REQUIREMENT])
            .map_err(|_| AppError::MissingDependencies())?;

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

        // get rpc url from command line or use default
        let rpc_url = match rpc_url {
            Some(url) => url,
            None => {
                print_warning!(
                    "No RPC URL provided, using default RPC URL: {}",
                    DEFAULT_RPC_URL
                );

                DEFAULT_RPC_URL.to_string()
            }
        };

        // all good, we can run the check
        let progress = create_spinner(&format!(
            "⏳ Checking contract for package {} at {}...",
            package_name,
            root.display()
        ));

        // run stylus check in contracts directory
        match self.stylus.check(&contracts_root, &rpc_url) {
            Ok(result) => {
                progress.finish_with_message(format!(
                    "{} Checked contract for package {} at {}\n",
                    "✅ Success!".green(),
                    package_name,
                    root.display()
                ));
                println!("{result}");
            }
            Err(e) => {
                progress.finish_with_message(format!(
                    "{} Checked contract for package {} at {}\n",
                    "❌ Error!".red(),
                    package_name,
                    root.display()
                ));
                return Err(AppError::StylusError(e.to_string()));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::requirements::MockTSystemRequirementsChecker;
    use crate::infrastructure::{nargo::MockTNargo, stylus::MockTStylus, system::MockTSystem};
    use mockall::predicate::*;
    use std::path::PathBuf;

    const RPC_URL: &str = "https://rpc.sepolia.org";
    const ROOT: &str = "circuit";
    const CONTRACTS_ROOT: &str = "circuit/contracts";
    const PACKAGE_NAME: &str = "hello_world";

    /// Happy path, package is provided
    #[tokio::test]
    async fn happy_path_with_package() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Ok(()));

        let mut nargo_mock = MockTNargo::new();
        nargo_mock
            .expect_find_package_root()
            .returning(|_| Ok(PathBuf::from(ROOT)));
        nargo_mock
            .expect_read_package_name()
            .returning(|_| Ok(PACKAGE_NAME.to_string()));

        let mut system_mock = MockTSystem::new();
        system_mock.expect_exists().returning(|_| true);

        let mut stylus_mock = MockTStylus::new();
        stylus_mock
            .expect_check()
            .returning(|_, _| Ok("✅ Success!".to_string()));

        let command = CheckCommand {
            system_requirements_checker: Box::new(system_requirements_checker_mock),
            stylus: Box::new(stylus_mock),
            system: Box::new(system_mock),
            nargo: Box::new(nargo_mock),
        };

        let result = command
            .run(
                &AppContext {},
                Some(PACKAGE_NAME.to_string()),
                Some(RPC_URL.to_string()),
            )
            .await;

        assert!(result.is_ok());
    }

    /// Happy path, no package is provided, we use current directory.
    #[tokio::test]
    async fn happy_path_without_package() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Ok(()));

        let mut nargo_mock = MockTNargo::new();
        nargo_mock
            .expect_read_package_name()
            .returning(|_| Ok(PACKAGE_NAME.to_string()));

        let mut system_mock = MockTSystem::new();
        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));
        system_mock.expect_exists().returning(|_| true);

        let mut stylus_mock = MockTStylus::new();
        stylus_mock
            .expect_check()
            .returning(|_, _| Ok("✅ Success!".to_string()));

        let command = CheckCommand {
            system_requirements_checker: Box::new(system_requirements_checker_mock),
            stylus: Box::new(stylus_mock),
            system: Box::new(system_mock),
            nargo: Box::new(nargo_mock),
        };

        let result = command
            .run(&AppContext {}, None, Some(RPC_URL.to_string()))
            .await;

        assert!(result.is_ok());
    }

    /// Should use default rpc url if not provided, based on
    #[tokio::test]
    async fn default_rpc_url() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Ok(()));

        let mut nargo_mock = MockTNargo::new();
        nargo_mock
            .expect_find_package_root()
            .returning(|_| Ok(PathBuf::from(ROOT)));
        nargo_mock
            .expect_read_package_name()
            .returning(|_| Ok(PACKAGE_NAME.to_string()));

        let mut system_mock = MockTSystem::new();
        system_mock.expect_exists().returning(|_| true);

        let mut stylus_mock = MockTStylus::new();
        stylus_mock
            .expect_check()
            .with(eq(PathBuf::from(CONTRACTS_ROOT)), eq(DEFAULT_RPC_URL.to_string()))
            .returning(|_, _| Ok("✅ Success!".to_string()));

        let command = CheckCommand {
            system_requirements_checker: Box::new(system_requirements_checker_mock),
            stylus: Box::new(stylus_mock),
            system: Box::new(system_mock),
            nargo: Box::new(nargo_mock),
        };

        let result = command
            .run(&AppContext {}, Some(PACKAGE_NAME.to_string()), None)
            .await;

        assert!(result.is_ok());
    }

    /// Should fail if dependencies are not met
    #[tokio::test]
    async fn missing_dependencies() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Err("cargo-stylus not found".to_string()));

        let command = CheckCommand {
            system_requirements_checker: Box::new(system_requirements_checker_mock),
            stylus: Box::new(MockTStylus::new()),
            system: Box::new(MockTSystem::new()),
            nargo: Box::new(MockTNargo::new()),
        };

        let result = command
            .run(&AppContext {}, Some(PACKAGE_NAME.to_string()), None)
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppError::MissingDependencies()
        ));
    }

    /// Should fail if specified package is not found
    #[tokio::test]
    async fn package_not_found() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Ok(()));

        let mut nargo_mock = MockTNargo::new();
        nargo_mock
            .expect_find_package_root()
            .returning(|_| Err("No package root found".into()));

        let command = CheckCommand {
            system_requirements_checker: Box::new(system_requirements_checker_mock),
            stylus: Box::new(MockTStylus::new()),
            system: Box::new(MockTSystem::new()),
            nargo: Box::new(nargo_mock),
        };

        let result = command
            .run(&AppContext {}, Some(PACKAGE_NAME.to_string()), None)
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppError::PackageNotFound);
    }

    /// Should fail if contracts directory is not found, generate should have created it.
    #[tokio::test]
    async fn contracts_directory_not_found() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Ok(()));

        let mut nargo_mock = MockTNargo::new();
        nargo_mock
            .expect_find_package_root()
            .returning(|_| Ok(PathBuf::from(ROOT)));
        nargo_mock
            .expect_read_package_name()
            .returning(|_| Ok(PACKAGE_NAME.to_string()));

        let mut system_mock = MockTSystem::new();
        system_mock.expect_exists().returning(|_| false); // contracts directory doesn't exist

        let command = CheckCommand {
            system_requirements_checker: Box::new(system_requirements_checker_mock),
            stylus: Box::new(MockTStylus::new()),
            system: Box::new(system_mock),
            nargo: Box::new(nargo_mock),
        };

        let result = command
            .run(&AppContext {}, Some(PACKAGE_NAME.to_string()), None)
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppError::ContractsNotFound(_)
        ));
    }

    /// Should fail if stylus check fails
    #[tokio::test]
    async fn stylus_failure() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Ok(()));

        let mut nargo_mock = MockTNargo::new();
        nargo_mock
            .expect_find_package_root()
            .returning(|_| Ok(PathBuf::from(ROOT)));
        nargo_mock
            .expect_read_package_name()
            .returning(|_| Ok(PACKAGE_NAME.to_string()));

        let mut system_mock = MockTSystem::new();
        system_mock.expect_exists().returning(|_| true);

        let mut stylus_mock = MockTStylus::new();
        stylus_mock
            .expect_check()
            .returning(|_, _| Err("stylus check failed".into()));

        let command = CheckCommand {
            system_requirements_checker: Box::new(system_requirements_checker_mock),
            stylus: Box::new(stylus_mock),
            system: Box::new(system_mock),
            nargo: Box::new(nargo_mock),
        };

        let result = command
            .run(&AppContext {}, Some(PACKAGE_NAME.to_string()), None)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::StylusError(_)));
    }
}
