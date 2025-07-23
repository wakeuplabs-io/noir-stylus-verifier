use crate::{
    config::{
        constants::DEFAULT_RPC_URL,
        requirements::{
            SystemRequirementsChecker, TSystemRequirementsChecker, BB_REQUIREMENT,
            BB_UP_REQUIREMENT,
        },
    },
    infrastructure::{
        bb::{Bb, TBb},
        progress::create_spinner,
        system::{System, TSystem},
    },
    print_error, print_warning, AppContext, AppError,
};
use alloy::{primitives::Bytes, providers::ProviderBuilder, sol};
use colored::*;
use std::path::PathBuf;

pub(crate) struct VerifyCommand {
    system: Box<dyn TSystem>,
    bb: Box<dyn TBb>,
    system_requirements_checker: Box<dyn TSystemRequirementsChecker>,
}

impl Default for VerifyCommand {
    fn default() -> Self {
        Self {
            system: Box::new(System),
            bb: Box::new(Bb::default()),
            system_requirements_checker: Box::new(SystemRequirementsChecker::default()),
        }
    }
}

sol! {
   #[sol(rpc)]
   contract Verifier {
        function verify(bytes proof, bytes public_inputs) public view returns (bool);
   }
}

impl VerifyCommand {
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        proof: Option<String>,
        public_input: Option<String>,
        vk: Option<String>,
        verifier_address: Option<String>,
        rpc_url: Option<String>,
        zk: bool,
    ) -> Result<(), AppError> {
        // verify dependencies
        self.system_requirements_checker
            .check(vec![BB_UP_REQUIREMENT])
            .map_err(|_| AppError::Other("Failed to verify dependencies"))?;

        // defaults to target folder
        let root = self.system.current_dir();
        let proof = PathBuf::from(proof.unwrap_or_else(|| {
            root.join("target")
                .join("proof")
                .to_string_lossy()
                .to_string()
        }));
        let public_input = PathBuf::from(public_input.unwrap_or_else(|| {
            root.join("target")
                .join("public_inputs")
                .to_string_lossy()
                .to_string()
        }));
        let vk = match vk {
            Some(vk) => PathBuf::from(vk),
            None => {
                let vk_path = root.join("contracts").join("assets").join("vk");
                if self.system.exists(&vk_path) {
                    vk_path
                } else if self.system.exists(&root.join("target").join("vk")) {
                    print_warning!("VK not found in contracts/assets, using ./target/vk instead");
                    root.join("target").join("vk")
                } else {
                    return Err(AppError::Other("VK not found"));
                }
            }
        };

        if !self.system.exists(&proof) {
            return Err(AppError::Other("Proof not found"));
        }
        if !self.system.exists(&public_input) {
            return Err(AppError::Other("Public input not found"));
        }
        if !self.system.exists(&vk) {
            return Err(AppError::Other("VK not found"));
        }

        // All good, let's verify the proof

        let spinner = create_spinner(&format!("⏳ Verifying proof at {}...", proof.display()));

        match verifier_address {
            Some(address) => {
                // call the verifier contract with the proof and public inputs
                let provider = ProviderBuilder::new().on_http(
                    rpc_url
                        .unwrap_or(DEFAULT_RPC_URL.to_string())
                        .parse()
                        .unwrap(),
                );

                let proof_bytes: Bytes = self.system.read_file(&proof).into();
                let public_input_bytes: Bytes = self.system.read_file(&public_input).into();
                let result = Verifier::new(address.parse().unwrap(), provider)
                    .verify(proof_bytes, public_input_bytes)
                    .call()
                    .await
                    .map_err(|e| AppError::RpcError(e.to_string()))?;

                if result._0 {
                    spinner.finish_with_message(format!(
                        "{} Proof verified onchain\n",
                        "✅ Success!".green(),
                    ));
                } else {
                    spinner.finish_with_message(format!(
                        "{} Proof verification failed onchain\n",
                        "❌ Error!".red(),
                    ));
                }
            }
            None => {
                self.bb
                    .setup(BB_REQUIREMENT.required_version)
                    .map_err(|_| AppError::Other("Failed to setup bb"))?;

                match self.bb.verify(&root, &proof, &public_input, &vk, zk) {
                    Ok(_) => {
                        spinner.finish_with_message(format!(
                            "{} Proof verified at {}\n",
                            "✅ Success!".green(),
                            root.join("target").join("proof").display()
                        ));
                    }
                    Err(e) => {
                        spinner.finish_with_message(format!(
                            "{} Failed to verify proof\n",
                            "❌ Error!".red()
                        ));
                        print_error!("{e}");
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::requirements::MockTSystemRequirementsChecker;
    use crate::infrastructure::{bb::MockTBb, system::MockTSystem};
    use mockall::predicate::*;
    use std::path::PathBuf;

    const ROOT: &str = "test";
    const PROOF_PATH: &str = "test/target/proof";
    const PUBLIC_INPUT_PATH: &str = "test/target/public_inputs";
    const VK_PATH: &str = "test/contracts/assets/vk";

    #[tokio::test]
    async fn test_verify_command_success_local() {
        // Test successful local verification using bb
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let mut system_mock = MockTSystem::new();
        let mut bb_mock = MockTBb::new();

        // Should check BB_UP_REQUIREMENT
        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == BB_UP_REQUIREMENT)
            .returning(|_| Ok(()));

        // System should return current directory
        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));

        // All files should exist
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(PROOF_PATH)))
            .returning(|_| true);
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(PUBLIC_INPUT_PATH)))
            .returning(|_| true);
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(VK_PATH)))
            .returning(|_| true);

        // BB should setup and verify successfully
        bb_mock
            .expect_setup()
            .with(eq(BB_REQUIREMENT.required_version))
            .returning(|_| Ok(()));
        bb_mock
            .expect_verify()
            .with(
                eq(PathBuf::from(ROOT)),
                eq(PathBuf::from(PROOF_PATH)),
                eq(PathBuf::from(PUBLIC_INPUT_PATH)),
                eq(PathBuf::from(VK_PATH)),
                eq(false),
            )
            .returning(|_, _, _, _, _| Ok(()));

        let command = VerifyCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command
            .run(
                &AppContext {},
                Some(PROOF_PATH.to_string()),
                Some(PUBLIC_INPUT_PATH.to_string()),
                Some(VK_PATH.to_string()),
                None, // no verifier_address
                None,
                false,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_command_success_local_zk() {
        // Test successful local verification using bb with zk flag
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let mut system_mock = MockTSystem::new();
        let mut bb_mock = MockTBb::new();

        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == BB_UP_REQUIREMENT)
            .returning(|_| Ok(()));

        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));

        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(PROOF_PATH)))
            .returning(|_| true);
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(PUBLIC_INPUT_PATH)))
            .returning(|_| true);
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(VK_PATH)))
            .returning(|_| true);

        bb_mock
            .expect_setup()
            .with(eq(BB_REQUIREMENT.required_version))
            .returning(|_| Ok(()));
        bb_mock
            .expect_verify()
            .with(
                eq(PathBuf::from(ROOT)),
                eq(PathBuf::from(PROOF_PATH)),
                eq(PathBuf::from(PUBLIC_INPUT_PATH)),
                eq(PathBuf::from(VK_PATH)),
                eq(true), // zk flag
            )
            .returning(|_, _, _, _, _| Ok(()));

        let command = VerifyCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command
            .run(
                &AppContext {},
                Some(PROOF_PATH.to_string()),
                Some(PUBLIC_INPUT_PATH.to_string()),
                Some(VK_PATH.to_string()),
                None,
                None,
                true, // zk flag
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_command_success_default_paths() {
        // Test successful verification with default file paths
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let mut system_mock = MockTSystem::new();
        let mut bb_mock = MockTBb::new();

        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == BB_UP_REQUIREMENT)
            .returning(|_| Ok(()));

        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));

        // Check default paths
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from("test/target/proof")))
            .returning(|_| true);
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from("test/target/public_inputs")))
            .returning(|_| true);
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from("test/contracts/assets/vk")))
            .returning(|_| true);

        bb_mock
            .expect_setup()
            .with(eq(BB_REQUIREMENT.required_version))
            .returning(|_| Ok(()));
        bb_mock
            .expect_verify()
            .with(
                eq(PathBuf::from(ROOT)),
                eq(PathBuf::from("test/target/proof")),
                eq(PathBuf::from("test/target/public_inputs")),
                eq(PathBuf::from("test/contracts/assets/vk")),
                eq(false),
            )
            .returning(|_, _, _, _, _| Ok(()));

        let command = VerifyCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command
            .run(
                &AppContext {},
                None, // use default
                None, // use default
                None, // use default
                None,
                None,
                false,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_command_success_fallback_vk() {
        // Test successful verification with fallback vk path (target/vk)
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let mut system_mock = MockTSystem::new();
        let mut bb_mock = MockTBb::new();

        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == BB_UP_REQUIREMENT)
            .returning(|_| Ok(()));

        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));

        system_mock
            .expect_exists()
            .with(eq(PathBuf::from("test/target/proof")))
            .returning(|_| true);
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from("test/target/public_inputs")))
            .returning(|_| true);
        // Primary vk path doesn't exist
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from("test/contracts/assets/vk")))
            .returning(|_| false);
        // Fallback vk path exists
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from("test/target/vk")))
            .returning(|_| true);

        bb_mock
            .expect_setup()
            .with(eq(BB_REQUIREMENT.required_version))
            .returning(|_| Ok(()));
        bb_mock
            .expect_verify()
            .with(
                eq(PathBuf::from(ROOT)),
                eq(PathBuf::from("test/target/proof")),
                eq(PathBuf::from("test/target/public_inputs")),
                eq(PathBuf::from("test/target/vk")), // fallback path
                eq(false),
            )
            .returning(|_, _, _, _, _| Ok(()));

        let command = VerifyCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command
            .run(&AppContext {}, None, None, None, None, None, false)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_command_missing_dependencies() {
        // Test failure when BB_UP_REQUIREMENT is not met
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let system_mock = MockTSystem::new();
        let bb_mock = MockTBb::new();

        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == BB_UP_REQUIREMENT)
            .returning(|_| Err("bbup not found".to_string()));

        let command = VerifyCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command
            .run(
                &AppContext {},
                Some(PROOF_PATH.to_string()),
                Some(PUBLIC_INPUT_PATH.to_string()),
                Some(VK_PATH.to_string()),
                None,
                None,
                false,
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Other(_)));
    }

    #[tokio::test]
    async fn test_verify_command_missing_proof() {
        // Test failure when proof file doesn't exist
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let mut system_mock = MockTSystem::new();
        let bb_mock = MockTBb::new();

        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == BB_UP_REQUIREMENT)
            .returning(|_| Ok(()));

        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));

        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(PROOF_PATH)))
            .returning(|_| false); // proof doesn't exist

        let command = VerifyCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command
            .run(
                &AppContext {},
                Some(PROOF_PATH.to_string()),
                Some(PUBLIC_INPUT_PATH.to_string()),
                Some(VK_PATH.to_string()),
                None,
                None,
                false,
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Other(_)));
    }

    #[tokio::test]
    async fn test_verify_command_missing_public_input() {
        // Test failure when public input file doesn't exist
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let mut system_mock = MockTSystem::new();
        let bb_mock = MockTBb::new();

        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == BB_UP_REQUIREMENT)
            .returning(|_| Ok(()));

        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));

        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(PROOF_PATH)))
            .returning(|_| true);
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(PUBLIC_INPUT_PATH)))
            .returning(|_| false); // public input doesn't exist

        let command = VerifyCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command
            .run(
                &AppContext {},
                Some(PROOF_PATH.to_string()),
                Some(PUBLIC_INPUT_PATH.to_string()),
                Some(VK_PATH.to_string()),
                None,
                None,
                false,
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Other(_)));
    }

    #[tokio::test]
    async fn test_verify_command_missing_vk() {
        // Test failure when VK file doesn't exist in any location
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let mut system_mock = MockTSystem::new();
        let bb_mock = MockTBb::new();

        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == BB_UP_REQUIREMENT)
            .returning(|_| Ok(()));

        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));

        system_mock
            .expect_exists()
            .with(eq(PathBuf::from("test/target/proof")))
            .returning(|_| true);
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from("test/target/public_inputs")))
            .returning(|_| true);
        // Both VK paths don't exist
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from("test/contracts/assets/vk")))
            .returning(|_| false);
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from("test/target/vk")))
            .returning(|_| false);

        let command = VerifyCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command
            .run(&AppContext {}, None, None, None, None, None, false)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Other(_)));
    }

    #[tokio::test]
    async fn test_verify_command_bb_setup_failure() {
        // Test failure when BB setup fails
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let mut system_mock = MockTSystem::new();
        let mut bb_mock = MockTBb::new();

        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == BB_UP_REQUIREMENT)
            .returning(|_| Ok(()));

        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));

        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(PROOF_PATH)))
            .returning(|_| true);
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(PUBLIC_INPUT_PATH)))
            .returning(|_| true);
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(VK_PATH)))
            .returning(|_| true);

        bb_mock
            .expect_setup()
            .with(eq(BB_REQUIREMENT.required_version))
            .returning(|_| Err("setup failed".into()));

        let command = VerifyCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command
            .run(
                &AppContext {},
                Some(PROOF_PATH.to_string()),
                Some(PUBLIC_INPUT_PATH.to_string()),
                Some(VK_PATH.to_string()),
                None,
                None,
                false,
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Other(_)));
    }

    #[tokio::test]
    async fn test_verify_command_bb_verify_failure() {
        // Test failure when BB verify fails
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        let mut system_mock = MockTSystem::new();
        let mut bb_mock = MockTBb::new();

        system_requirements_checker_mock
            .expect_check()
            .withf(|reqs| reqs.len() == 1 && reqs[0] == BB_UP_REQUIREMENT)
            .returning(|_| Ok(()));

        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));

        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(PROOF_PATH)))
            .returning(|_| true);
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(PUBLIC_INPUT_PATH)))
            .returning(|_| true);
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(VK_PATH)))
            .returning(|_| true);

        bb_mock
            .expect_setup()
            .with(eq(BB_REQUIREMENT.required_version))
            .returning(|_| Ok(()));
        bb_mock
            .expect_verify()
            .with(
                eq(PathBuf::from(ROOT)),
                eq(PathBuf::from(PROOF_PATH)),
                eq(PathBuf::from(PUBLIC_INPUT_PATH)),
                eq(PathBuf::from(VK_PATH)),
                eq(false),
            )
            .returning(|_, _, _, _, _| Err("verification failed".into()));

        let command = VerifyCommand {
            system: Box::new(system_mock),
            bb: Box::new(bb_mock),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command
            .run(
                &AppContext {},
                Some(PROOF_PATH.to_string()),
                Some(PUBLIC_INPUT_PATH.to_string()),
                Some(VK_PATH.to_string()),
                None,
                None,
                false,
            )
            .await;

        // Should succeed (returns Ok(())) but prints error internally
        assert!(result.is_ok());
    }

    // Note: On-chain verification tests would require mocking the alloy provider
    // which is more complex and would need additional setup. The current tests
    // cover the core logic paths for the verify command.
}
