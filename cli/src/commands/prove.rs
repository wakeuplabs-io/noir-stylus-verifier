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

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::requirements::MockTSystemRequirementsChecker;
    use crate::infrastructure::{bb::MockTBb, nargo::MockTNargo, system::MockTSystem};
    use mockall::predicate::*;
    use std::path::PathBuf;

    const ROOT: &str = "test";
    const PACKAGE_NAME: &str = "test";

    #[tokio::test]
    async fn test_prove_command_success_with_package() {
        // Test successful proof generation with package name provided
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let mut system_mock = MockTSystem::new();
        let mut bb_mock = MockTBb::new();
        let mut nargo_mock = MockTNargo::new();

        // we use noirup and bbup to ensure proper versions
        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| {
                reqs.len() == 2 && reqs[0] == BB_UP_REQUIREMENT && reqs[1] == NOIRUP_REQUIREMENT
            })
            .returning(|_| Ok(()));

        // identify package root and name
        nargo_mock
            .expect_find_package_root()
            .with(eq(PACKAGE_NAME))
            .returning(|_| Ok(PathBuf::from(ROOT)));
        nargo_mock
            .expect_read_package_name()
            .with(eq(PathBuf::from(ROOT)))
            .returning(|_| Ok(PACKAGE_NAME.to_string()));

        // we provided a package name so shouldn't use current dir to determine root
        system_mock.expect_current_dir().never();

        // execute nargo and bb
        nargo_mock
            .expect_execute()
            .with(eq(PathBuf::from(ROOT)), eq(PACKAGE_NAME))
            .returning(|_, _| Ok(()));
        bb_mock
            .expect_prove()
            .with(eq(PathBuf::from(ROOT)), eq(PACKAGE_NAME), eq(false))
            .returning(|_, _, _| Ok(()));

        let command = ProveCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            nargo: Box::new(nargo_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command
            .run(&AppContext {}, Some(PACKAGE_NAME.to_string()), false)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_prove_command_success_with_package_zk() {
        // Test successful proof generation with package name provided and zk flag
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let mut system_mock = MockTSystem::new();
        let mut bb_mock = MockTBb::new();
        let mut nargo_mock = MockTNargo::new();

        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| {
                reqs.len() == 2 && reqs[0] == BB_UP_REQUIREMENT && reqs[1] == NOIRUP_REQUIREMENT
            })
            .returning(|_| Ok(()));

        nargo_mock
            .expect_find_package_root()
            .with(eq(PACKAGE_NAME))
            .returning(|_| Ok(PathBuf::from(ROOT)));
        nargo_mock
            .expect_read_package_name()
            .with(eq(PathBuf::from(ROOT)))
            .returning(|_| Ok(PACKAGE_NAME.to_string()));

        system_mock.expect_current_dir().never();

        nargo_mock
            .expect_execute()
            .with(eq(PathBuf::from(ROOT)), eq(PACKAGE_NAME))
            .returning(|_, _| Ok(()));
        bb_mock
            .expect_prove()
            .with(eq(PathBuf::from(ROOT)), eq(PACKAGE_NAME), eq(true)) // zk flag true
            .returning(|_, _, _| Ok(()));

        let command = ProveCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            nargo: Box::new(nargo_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command
            .run(&AppContext {}, Some(PACKAGE_NAME.to_string()), true)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_prove_command_success_without_package() {
        // Test successful proof generation without package name (uses current dir)
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let mut system_mock = MockTSystem::new();
        let mut bb_mock = MockTBb::new();
        let mut nargo_mock = MockTNargo::new();

        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| {
                reqs.len() == 2 && reqs[0] == BB_UP_REQUIREMENT && reqs[1] == NOIRUP_REQUIREMENT
            })
            .returning(|_| Ok(()));

        // No package provided, so should use current dir
        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));

        // Should not call find_package_root since no package was provided
        nargo_mock.expect_find_package_root().never();

        nargo_mock
            .expect_read_package_name()
            .with(eq(PathBuf::from(ROOT)))
            .returning(|_| Ok(PACKAGE_NAME.to_string()));

        nargo_mock
            .expect_execute()
            .with(eq(PathBuf::from(ROOT)), eq(PACKAGE_NAME))
            .returning(|_, _| Ok(()));
        bb_mock
            .expect_prove()
            .with(eq(PathBuf::from(ROOT)), eq(PACKAGE_NAME), eq(false))
            .returning(|_, _, _| Ok(()));

        let command = ProveCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            nargo: Box::new(nargo_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command.run(&AppContext {}, None, false).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_prove_command_missing_dependencies() {
        // Test failure when dependencies are not met
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let system_mock = MockTSystem::new();
        let bb_mock = MockTBb::new();
        let nargo_mock = MockTNargo::new();

        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| {
                reqs.len() == 2 && reqs[0] == BB_UP_REQUIREMENT && reqs[1] == NOIRUP_REQUIREMENT
            })
            .returning(|_| Err("dependencies not found".to_string()));

        let command = ProveCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            nargo: Box::new(nargo_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command
            .run(&AppContext {}, Some(PACKAGE_NAME.to_string()), false)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Other(_)));
    }

    #[tokio::test]
    async fn test_prove_command_package_not_found() {
        // Test failure when package is not found
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let system_mock = MockTSystem::new();
        let bb_mock = MockTBb::new();
        let mut nargo_mock = MockTNargo::new();

        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| {
                reqs.len() == 2 && reqs[0] == BB_UP_REQUIREMENT && reqs[1] == NOIRUP_REQUIREMENT
            })
            .returning(|_| Ok(()));

        nargo_mock
            .expect_find_package_root()
            .with(eq(PACKAGE_NAME))
            .returning(|_| Err("package not found".into()));

        let command = ProveCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            nargo: Box::new(nargo_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command
            .run(&AppContext {}, Some(PACKAGE_NAME.to_string()), false)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::PackageNotFound));
    }

    #[tokio::test]
    async fn test_prove_command_read_package_name_failure() {
        // Test failure when reading package name fails
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let mut system_mock = MockTSystem::new();
        let bb_mock = MockTBb::new();
        let mut nargo_mock = MockTNargo::new();

        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| {
                reqs.len() == 2 && reqs[0] == BB_UP_REQUIREMENT && reqs[1] == NOIRUP_REQUIREMENT
            })
            .returning(|_| Ok(()));

        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));

        nargo_mock
            .expect_read_package_name()
            .with(eq(PathBuf::from(ROOT)))
            .returning(|_| Err("failed to read package name".into()));

        let command = ProveCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            nargo: Box::new(nargo_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command.run(&AppContext {}, None, false).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::PackageNotFound));
    }

    #[tokio::test]
    async fn test_prove_command_nargo_execute_failure() {
        // Test failure when nargo execution fails
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let mut system_mock = MockTSystem::new();
        let bb_mock = MockTBb::new();
        let mut nargo_mock = MockTNargo::new();

        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| {
                reqs.len() == 2 && reqs[0] == BB_UP_REQUIREMENT && reqs[1] == NOIRUP_REQUIREMENT
            })
            .returning(|_| Ok(()));

        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));

        nargo_mock
            .expect_read_package_name()
            .with(eq(PathBuf::from(ROOT)))
            .returning(|_| Ok(PACKAGE_NAME.to_string()));

        nargo_mock
            .expect_execute()
            .with(eq(PathBuf::from(ROOT)), eq(PACKAGE_NAME))
            .returning(|_, _| Err("nargo execution failed".into()));

        let command = ProveCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            nargo: Box::new(nargo_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command.run(&AppContext {}, None, false).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Other(_)));
    }

    #[tokio::test]
    async fn test_prove_command_bb_prove_failure() {
        // Test failure when bb prove fails
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let mut system_mock = MockTSystem::new();
        let mut bb_mock = MockTBb::new();
        let mut nargo_mock = MockTNargo::new();

        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| {
                reqs.len() == 2 && reqs[0] == BB_UP_REQUIREMENT && reqs[1] == NOIRUP_REQUIREMENT
            })
            .returning(|_| Ok(()));

        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));

        nargo_mock
            .expect_read_package_name()
            .with(eq(PathBuf::from(ROOT)))
            .returning(|_| Ok(PACKAGE_NAME.to_string()));

        nargo_mock
            .expect_execute()
            .with(eq(PathBuf::from(ROOT)), eq(PACKAGE_NAME))
            .returning(|_, _| Ok(()));

        bb_mock
            .expect_prove()
            .with(eq(PathBuf::from(ROOT)), eq(PACKAGE_NAME), eq(false))
            .returning(|_, _, _| Err("bb prove failed".into()));

        let command = ProveCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            nargo: Box::new(nargo_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command.run(&AppContext {}, None, false).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Other(_)));
    }

    #[tokio::test]
    async fn test_prove_command_success_zk_no_package() {
        // Test successful proof generation with zk flag and no package name
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let mut system_mock = MockTSystem::new();
        let mut bb_mock = MockTBb::new();
        let mut nargo_mock = MockTNargo::new();

        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| {
                reqs.len() == 2 && reqs[0] == BB_UP_REQUIREMENT && reqs[1] == NOIRUP_REQUIREMENT
            })
            .returning(|_| Ok(()));

        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));

        nargo_mock
            .expect_read_package_name()
            .with(eq(PathBuf::from(ROOT)))
            .returning(|_| Ok(PACKAGE_NAME.to_string()));

        nargo_mock
            .expect_execute()
            .with(eq(PathBuf::from(ROOT)), eq(PACKAGE_NAME))
            .returning(|_, _| Ok(()));
        bb_mock
            .expect_prove()
            .with(eq(PathBuf::from(ROOT)), eq(PACKAGE_NAME), eq(true)) // zk flag true
            .returning(|_, _, _| Ok(()));

        let command = ProveCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            nargo: Box::new(nargo_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command.run(&AppContext {}, None, true).await;

        assert!(result.is_ok());
    }
}
