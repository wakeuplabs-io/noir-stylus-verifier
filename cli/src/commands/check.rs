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

    // default values for testing
    const RPC_URL: &str = "https://rpc.sepolia.org";
    const ROOT: &str = "circuit";
    const PACKAGE_NAME: &str = "hello_world";

    /// Basic test case, user provides all parameters.
    /// We test we properly run verifications and call the deployment with the correct parameters.
    #[tokio::test]
    async fn test_check_command() {
        // should check we have stylus installed
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == CARGO_STYLUS_REQUIREMENT)
            .returning(|_| Ok(()));

        let mut nargo_mock = MockTNargo::new();
        nargo_mock
            .expect_find_package_root()
            .with(eq(PACKAGE_NAME))
            .returning(|_| Ok(PathBuf::from(ROOT)));
        nargo_mock
            .expect_read_package_name()
            .with(eq(PathBuf::from(ROOT)))
            .returning(|_| Ok(PACKAGE_NAME.to_string()));

        // should check we're at the circuit root
        let mut system_mock = MockTSystem::new();
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(ROOT).join("contracts")))
            .returning(|_| true);

        // should run stylus check
        let mut stylus_mock = MockTStylus::new();
        stylus_mock
            .expect_check()
            .with(eq(PathBuf::from(ROOT).join("contracts")), eq(RPC_URL))
            .returning(|_, _| Ok("✅ Success!".to_string()));

        let result = CheckCommand {
            system_requirements_checker: Box::new(system_requirements_checker_mock),
            stylus: Box::new(stylus_mock),
            system: Box::new(system_mock),
            nargo: Box::new(nargo_mock),
        }
        .run(
            &AppContext {},
            Some(PACKAGE_NAME.to_string()),
            Some(RPC_URL.to_string()),
        )
        .await;

        assert!(result.is_ok());
    }

    /// If user does not provide rpc url, we should use the default one.
    /// This test checks that we properly determine the default rpc url based on the rpc provided.
    #[tokio::test]
    async fn check_uses_sepolia_as_default_rpc_url() {
        // should check we have stylus installed
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == CARGO_STYLUS_REQUIREMENT)
            .returning(|_| Ok(()));

        let mut nargo_mock = MockTNargo::new();
        nargo_mock
            .expect_find_package_root()
            .with(eq(PACKAGE_NAME))
            .returning(|_| Ok(PathBuf::from(ROOT)));
        nargo_mock
            .expect_read_package_name()
            .with(eq(PathBuf::from(ROOT)))
            .returning(|_| Ok(PACKAGE_NAME.to_string()));

        // should check we're at the circuit root
        let mut system_mock = MockTSystem::new();
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(ROOT).join("contracts")))
            .returning(|_| true);

        // should run stylus check
        let mut stylus_mock = MockTStylus::new();
        stylus_mock
            .expect_check()
            .with(
                eq(PathBuf::from(ROOT).join("contracts")),
                eq(DEFAULT_RPC_URL),
            )
            .returning(|_, _| Ok("✅ Success!".to_string()));

        let result = CheckCommand {
            system_requirements_checker: Box::new(system_requirements_checker_mock),
            stylus: Box::new(stylus_mock),
            system: Box::new(system_mock),
            nargo: Box::new(nargo_mock),
        }
        .run(&AppContext {}, Some(PACKAGE_NAME.to_string()), None)
        .await;

        assert!(result.is_ok());
    }

    /// When we can't find the package root, we should fail right away.
    #[tokio::test]
    async fn check_package_root_fails() {
        // should check we have stylus installed
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == CARGO_STYLUS_REQUIREMENT)
            .returning(|_| Ok(()));

        // find package root
        let mut nargo_mock = MockTNargo::new();
        nargo_mock
            .expect_find_package_root()
            .with(eq(PACKAGE_NAME))
            .returning(|_| Err("No package root found".into()));
        nargo_mock.expect_read_package_name().never();

        // Check contracts were already generated
        let mut system_mock = MockTSystem::new();
        system_mock.expect_exists().never();

        // should not run stylus check if we can't find the contracts directory
        let mut stylus_mock = MockTStylus::new();
        stylus_mock.expect_check().never();

        let result = CheckCommand {
            system_requirements_checker: Box::new(system_requirements_checker_mock),
            stylus: Box::new(stylus_mock),
            system: Box::new(system_mock),
            nargo: Box::new(nargo_mock),
        }
        .run(&AppContext {}, Some(PACKAGE_NAME.to_string()), None)
        .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppError::PackageNotFound);
    }
}
