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
            .execute(&root, &package_name)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::requirements::MockTSystemRequirementsChecker;
    use crate::infrastructure::{bb::MockTBb, nargo::MockTNargo, system::MockTSystem};
    use std::path::PathBuf;

    const ROOT: &str = "test";
    const PACKAGE_NAME: &str = "test";

    /// Happy path, package is provided
    #[tokio::test]
    async fn success_with_package() {
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

        let mut bb_mock = MockTBb::new();
        bb_mock.expect_prove().returning(|_, _, _| Ok(()));

        nargo_mock.expect_execute().returning(|_, _| Ok(()));

        let command = ProveCommand {
            system: Box::new(MockTSystem::new()),
            bb: Box::new(bb_mock),
            nargo: Box::new(nargo_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command
            .run(&AppContext {}, Some(PACKAGE_NAME.to_string()), false)
            .await;

        assert!(result.is_ok());
    }

    /// Happy path, package is provided and zk flavour is enabled
    #[tokio::test]
    async fn success_with_package_zk() {
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

        let mut bb_mock = MockTBb::new();
        bb_mock.expect_prove().returning(|_, _, _| Ok(()));

        nargo_mock.expect_execute().returning(|_, _| Ok(()));

        let command = ProveCommand {
            system: Box::new(MockTSystem::new()),
            bb: Box::new(bb_mock),
            nargo: Box::new(nargo_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command
            .run(&AppContext {}, Some(PACKAGE_NAME.to_string()), true)
            .await;

        assert!(result.is_ok());
    }

    /// Should default to current directory if no package is provided
    #[tokio::test]
    async fn success_without_package() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Ok(()));

        let mut system_mock = MockTSystem::new();
        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));

        let mut nargo_mock = MockTNargo::new();
        nargo_mock
            .expect_read_package_name()
            .returning(|_| Ok(PACKAGE_NAME.to_string()));

        let mut bb_mock = MockTBb::new();
        bb_mock.expect_prove().returning(|_, _, _| Ok(()));

        nargo_mock.expect_execute().returning(|_, _| Ok(()));

        let command = ProveCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            nargo: Box::new(nargo_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command.run(&AppContext {}, None, false).await;

        assert!(result.is_ok());
    }

    /// Should fail if dependencies are not met
    #[tokio::test]
    async fn missing_dependencies() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Err("dependencies not found".to_string()));

        let command = ProveCommand {
            system: Box::new(MockTSystem::new()),
            bb: Box::new(MockTBb::new()),
            nargo: Box::new(MockTNargo::new()),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command
            .run(&AppContext {}, Some(PACKAGE_NAME.to_string()), false)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Other(_)));
    }

    /// Should fail if package is not found
    #[tokio::test]
    async fn package_not_found() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Ok(()));

        let mut nargo_mock = MockTNargo::new();
        nargo_mock
            .expect_find_package_root()
            .returning(|_| Err("package not found".into()));

        let command = ProveCommand {
            system: Box::new(MockTSystem::new()),
            bb: Box::new(MockTBb::new()),
            nargo: Box::new(nargo_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command
            .run(&AppContext {}, Some(PACKAGE_NAME.to_string()), false)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::PackageNotFound));
    }
}
