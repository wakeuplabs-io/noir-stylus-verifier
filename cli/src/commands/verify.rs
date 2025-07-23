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
    use std::path::PathBuf;

    const ROOT: &str = "test";
    const PROOF_PATH: &str = "test/target/proof";
    const PUBLIC_INPUT_PATH: &str = "test/target/public_inputs";
    const VK_PATH: &str = "test/contracts/assets/vk";

    /// Happy path, local verification
    #[tokio::test]
    async fn success_local() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Ok(()));

        let mut system_mock = MockTSystem::new();
        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));
        system_mock.expect_exists().returning(|_| true);

        let mut bb_mock = MockTBb::new();
        bb_mock.expect_setup().returning(|_| Ok(()));
        bb_mock.expect_verify().returning(|_, _, _, _, _| Ok(()));

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

        assert!(result.is_ok());
    }

    /// Happy path, local verification with zk flavour
    #[tokio::test]
    async fn success_local_zk() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Ok(()));

        let mut system_mock = MockTSystem::new();
        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));
        system_mock.expect_exists().returning(|_| true);

        let mut bb_mock = MockTBb::new();
        bb_mock.expect_setup().returning(|_| Ok(()));
        bb_mock.expect_verify().returning(|_, _, _, _, _| Ok(()));

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

    /// Happy path, default paths are used
    #[tokio::test]
    async fn success_default_paths() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Ok(()));

        let mut system_mock = MockTSystem::new();
        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));
        system_mock.expect_exists().returning(|_| true);

        let mut bb_mock = MockTBb::new();
        bb_mock.expect_setup().returning(|_| Ok(()));
        bb_mock.expect_verify().returning(|_, _, _, _, _| Ok(()));

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

    /// Happy path, fallback vk path is used
    #[tokio::test]
    async fn success_fallback_vk() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Ok(()));

        let mut system_mock = MockTSystem::new();
        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));
        system_mock.expect_exists().returning(|path| {
            // Primary vk path doesn't exist, fallback does
            !path.to_string_lossy().contains("contracts/assets/vk")
        });

        let mut bb_mock = MockTBb::new();
        bb_mock.expect_setup().returning(|_| Ok(()));
        bb_mock.expect_verify().returning(|_, _, _, _, _| Ok(()));

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

    /// Should fail if dependencies are not met
    #[tokio::test]
    async fn missing_dependencies() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Err("bbup not found".to_string()));

        let command = VerifyCommand {
            system: Box::new(MockTSystem::new()),
            bb: Box::new(MockTBb::new()),
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

    /// Should fail if proof is not found
    #[tokio::test]
    async fn missing_proof() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Ok(()));

        let mut system_mock = MockTSystem::new();
        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));
        system_mock
            .expect_exists()
            .returning(|path| !path.to_string_lossy().contains("proof"));

        let command = VerifyCommand {
            system: Box::new(system_mock),
            bb: Box::new(MockTBb::new()),
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

    /// Should fail if public input is not found
    #[tokio::test]
    async fn missing_public_input() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Ok(()));

        let mut system_mock = MockTSystem::new();
        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));
        system_mock
            .expect_exists()
            .returning(|path| !path.to_string_lossy().contains("public_inputs"));

        let command = VerifyCommand {
            system: Box::new(system_mock),
            bb: Box::new(MockTBb::new()),
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

    /// Should fail if vk is not found
    #[tokio::test]
    async fn missing_vk() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Ok(()));

        let mut system_mock = MockTSystem::new();
        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));
        system_mock
            .expect_exists()
            .returning(|path| !path.to_string_lossy().contains("vk"));

        let command = VerifyCommand {
            system: Box::new(system_mock),
            bb: Box::new(MockTBb::new()),
            system_requirements_checker: Box::new(system_requirements_checker_mock),
        };

        let result = command
            .run(&AppContext {}, None, None, None, None, None, false)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Other(_)));
    }

    /// Should fail if bb setup fails, we need specific version of bb for compatibility
    #[tokio::test]
    async fn bb_setup_failure() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Ok(()));

        let mut system_mock = MockTSystem::new();
        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));
        system_mock.expect_exists().returning(|_| true);

        let mut bb_mock = MockTBb::new();
        bb_mock
            .expect_setup()
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

    /// Proof verification fails
    #[tokio::test]
    async fn bb_verify_failure() {
        let mut system_requirements_checker_mock = MockTSystemRequirementsChecker::new();
        system_requirements_checker_mock
            .expect_check()
            .returning(|_| Ok(()));

        let mut system_mock = MockTSystem::new();
        system_mock
            .expect_current_dir()
            .returning(|| PathBuf::from(ROOT));
        system_mock.expect_exists().returning(|_| true);

        let mut bb_mock = MockTBb::new();
        bb_mock.expect_setup().returning(|_| Ok(()));
        bb_mock
            .expect_verify()
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
}
