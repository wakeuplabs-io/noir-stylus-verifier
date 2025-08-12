//! # Nargo Integration
//!
//! This module provides integration with the Nargo CLI, which is the official
//! toolchain for the Noir programming language. It handles project discovery,
//! circuit compilation, and witness generation for Noir circuits.

use crate::infrastructure::system::{System, TSystem};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// Trait defining the interface for Nargo operations.
/// 
/// This trait abstracts interactions with the Nargo CLI tool, enabling
/// project management, compilation, and execution of Noir circuits.
#[cfg_attr(test, mockall::automock)]
pub(crate) trait TNargo {
    /// Finds the root directory of a Noir package by name.
    /// 
    /// Searches up the directory tree from the current working directory
    /// looking for a `Nargo.toml` file with a matching package name.
    /// 
    /// # Arguments
    /// 
    /// * `package` - The name of the package to find
    /// 
    /// # Returns
    /// 
    /// Returns the path to the package root directory, or an error if
    /// the package is not found within the search scope.
    fn find_package_root(&self, package: &str) -> Result<PathBuf, Box<dyn std::error::Error>>;
    
    /// Reads the package name from a Nargo.toml file.
    /// 
    /// Parses the TOML configuration file to extract the package name
    /// from the `[package]` section.
    /// 
    /// # Arguments
    /// 
    /// * `root` - Path to the directory containing Nargo.toml
    /// 
    /// # Returns
    /// 
    /// Returns the package name as a string, or an error if the file
    /// cannot be read or the package name is not found.
    fn read_package_name(&self, root: &Path) -> Result<String, Box<dyn std::error::Error>>;
    
    /// Executes a Noir circuit to generate a witness.
    /// 
    /// Runs `nargo execute` with the specified package and prover configuration,
    /// which evaluates the circuit with the provided inputs to generate a witness.
    /// 
    /// # Arguments
    /// 
    /// * `root` - Project root directory for command execution
    /// * `package_name` - Name of the package to execute
    /// * `prover_name` - Name of the prover configuration to use
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` if execution succeeds, or an error if the
    /// nargo command fails or the witness generation fails.
    fn execute(
        &self,
        root: &Path,
        package_name: &str,
        prover_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Compiles a Noir circuit to bytecode.
    /// 
    /// Runs `nargo compile` to generate circuit bytecode, which is then
    /// copied to the specified output path for use in proof generation.
    /// 
    /// # Arguments
    /// 
    /// * `root` - Project root directory for command execution
    /// * `package_name` - Name of the package to compile
    /// * `bytecode_path` - Path where the compiled bytecode should be written
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` if compilation succeeds, or an error if the
    /// nargo command fails or file operations fail.
    fn compile(
        &self,
        root: &Path,
        package_name: &str,
        bytecode_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

/// Nargo CLI integration for Noir circuit operations.
/// 
/// This struct provides a wrapper around the Nargo command-line tool,
/// enabling project discovery, compilation, and execution of Noir circuits.
pub(crate) struct Nargo {
    /// System interface for executing nargo commands and file operations
    system: Box<dyn TSystem>,
}

impl Default for Nargo {
    fn default() -> Self {
        Self {
            system: Box::new(System),
        }
    }
}

impl TNargo for Nargo {
    /// Implements package discovery by searching up the directory tree.
    /// 
    /// Starting from the current directory, searches up to 5 parent directories
    /// for a `Nargo.toml` file with a matching package name.
    fn find_package_root(&self, package: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let mut current_dir = self.system.current_dir();

        // Look up to 5 parent directories
        for _ in 0..5 {
            if self.system.exists(&current_dir.join("Nargo.toml")) {
                // check if package name is correct
                let package_name = self.read_package_name(&current_dir)?;
                if package_name == package {
                    return Ok(current_dir);
                }
            }

            // Try going up one directory level
            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                break;
            }
        }

        Err(format!("Circuit not found at or above: {package}").into())
    }

    /// Implements package name reading from Nargo.toml.
    /// 
    /// Parses the TOML file and extracts the package name from the
    /// `[package].name` field.
    fn read_package_name(&self, root: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let content = self.system.read_file_str(&root.join("Nargo.toml"));
        let toml: toml::Value = toml::from_str(&content)?;

        if let Some(name) = toml
            .get("package")
            .and_then(|pkg| pkg.get("name"))
            .and_then(toml::Value::as_str)
        {
            Ok(name.to_string())
        } else {
            Err(format!("Package name not found in Nargo.toml at {}", root.display()).into())
        }
    }

    /// Implements circuit compilation using the nargo CLI.
    /// 
    /// Executes `nargo compile` to generate bytecode, then copies the
    /// resulting file from the target directory to the specified output path.
    fn compile(
        &self,
        root: &Path,
        package_name: &str,
        bytecode_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.system
            .execute_command(Command::new("nargo").arg("compile").current_dir(root))?;

        self.system.copy_file(
            &root.join("target").join(format!("{package_name}.json")),
            bytecode_path,
        );

        Ok(())
    }

    /// Implements circuit execution using the nargo CLI.
    /// 
    /// Executes `nargo execute` with the specified package and prover
    /// configuration to generate a witness from the circuit inputs.
    fn execute(
        &self,
        root: &Path,
        package_name: &str,
        prover_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.system.execute_command(
            Command::new("nargo")
                .arg("execute")
                .arg("--package")
                .arg(package_name)
                .arg("--prover-name")
                .arg(prover_name)
                .current_dir(root),
        )?;

        Ok(())
    }
}
