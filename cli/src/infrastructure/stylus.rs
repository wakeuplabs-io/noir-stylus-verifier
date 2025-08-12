//! # Stylus CLI Integration
//!
//! This module provides integration with the cargo-stylus CLI tool for managing
//! Stylus contracts. It handles contract deployment, compatibility checking, and
//! gas estimation for Arbitrum Stylus smart contracts.

use std::{path::Path, process::Command};

use crate::infrastructure::system::{System, TSystem};

/// Stylus CLI integration for contract operations.
///
/// This struct provides a wrapper around the cargo-stylus command-line tool,
/// enabling deployment and validation of Stylus smart contracts on Arbitrum.
pub(crate) struct Stylus {
    /// System interface for executing cargo-stylus commands
    system: Box<dyn TSystem>,
}

/// Trait defining the interface for Stylus operations.
///
/// This trait abstracts interactions with the cargo-stylus CLI tool,
/// enabling contract deployment and validation operations.
#[cfg_attr(test, mockall::automock)]
pub(crate) trait TStylus: Send + Sync {
    /// Deploys a Stylus contract to the blockchain.
    ///
    /// Executes `cargo stylus deploy` to compile and deploy a Stylus contract
    /// with the specified configuration and constructor arguments.
    ///
    /// # Arguments
    ///
    /// * `root` - Path to the contract directory containing Cargo.toml
    /// * `rpc_url` - RPC endpoint for the target blockchain
    /// * `private_key` - Private key for signing the deployment transaction
    /// * `constructor_args` - Constructor arguments for contract initialization
    ///
    /// # Returns
    ///
    /// Returns the deployment output including contract address and transaction
    /// details, or an error if deployment fails.
    fn deploy(
        &self,
        root: &Path,
        rpc_url: &str,
        private_key: &str,
        constructor_args: &str,
    ) -> Result<String, Box<dyn std::error::Error>>;

    /// Checks Stylus contract compatibility and estimates deployment costs.
    ///
    /// Executes `cargo stylus check` to validate that the contract is compatible
    /// with the Stylus runtime and provides gas cost estimates for deployment.
    ///
    /// # Arguments
    ///
    /// * `root` - Path to the contract directory containing Cargo.toml
    /// * `rpc_url` - RPC endpoint for compatibility checking
    ///
    /// # Returns
    ///
    /// Returns the check output including compatibility status and gas estimates,
    /// or an error if the check fails or the contract is incompatible.
    fn check(&self, root: &Path, rpc_url: &str) -> Result<String, Box<dyn std::error::Error>>;
}

// implementations ==========================================

impl Default for Stylus {
    fn default() -> Self {
        Self {
            system: Box::new(System),
        }
    }
}

impl TStylus for Stylus {
    /// Implements contract deployment using the cargo-stylus CLI.
    ///
    /// Executes `cargo stylus deploy` with the no-verify flag to deploy
    /// the contract without additional verification steps.
    fn deploy(
        &self,
        root: &Path,
        rpc_url: &str,
        private_key: &str,
        constructor_args: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let result = self.system.execute_command(
            Command::new("cargo")
                .arg("stylus")
                .arg("deploy")
                .arg("--no-verify")
                .arg("--endpoint")
                .arg(rpc_url)
                .arg("--private-key")
                .arg(private_key)
                .arg("--constructor-args")
                .arg(constructor_args)
                .current_dir(root),
        )?;

        Ok(result)
    }

    /// Implements contract compatibility checking using the cargo-stylus CLI.
    ///
    /// Executes `cargo stylus check` to validate contract compatibility
    /// and provide deployment cost estimates.
    fn check(&self, root: &Path, rpc_url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let result = self.system.execute_command(
            Command::new("cargo")
                .arg("stylus")
                .arg("check")
                .arg("-e")
                .arg(rpc_url)
                .current_dir(root),
        )?;

        Ok(result)
    }
}
