//! # Barretenberg Wrapper
//!
//! This module is a wrapper around the Barretenberg (bb) cli.

use crate::infrastructure::system::{System, TSystem};
use std::{path::Path, process::Command};

/// Barretenberg backend integration for cryptographic operations.
pub(crate) struct Bb {
    /// System interface for executing bb commands and file operations
    system: Box<dyn TSystem>,
}

/// Trait defining the interface for Barretenberg operations.
/// 
/// This trait abstracts the core cryptographic operations provided by the
/// Barretenberg backend, enabling easy testing and dependency injection.
#[cfg_attr(test, mockall::automock)]
pub(crate) trait TBb {
    /// Generates a verification key from compiled circuit bytecode.
    /// 
    /// # Arguments
    /// 
    /// * `root` - Project root directory for command execution
    /// * `bytecode` - Path to the compiled circuit bytecode file
    /// * `output` - Path where the verification key should be written
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` if the verification key is generated successfully,
    /// or an error if the bb command fails or file operations fail.
    fn write_vk(
        &self,
        root: &Path,
        bytecode: &Path,
        output: &Path,
    ) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Generates a cryptographic proof for a circuit.
    /// 
    /// # Arguments
    /// 
    /// * `root` - Project root directory for command execution
    /// * `bytecode` - Path to the compiled circuit bytecode
    /// * `witness` - Path to the circuit witness file
    /// * `output` - Directory where proof and public inputs will be written
    /// * `zk` - Whether to generate a zero-knowledge proof
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` if proof generation succeeds, or an error if the
    /// bb command fails or file operations fail.
    fn prove(
        &self,
        root: &Path,
        bytecode: &Path,
        witness: &Path,
        output: &Path,
        zk: bool,
    ) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Verifies a cryptographic proof locally using Barretenberg.
    /// 
    /// # Arguments
    /// 
    /// * `root` - Project root directory for command execution
    /// * `proof` - Path to the proof file to verify
    /// * `public_input` - Path to the public inputs file
    /// * `vk` - Path to the verification key file
    /// * `zk` - Whether this is a zero-knowledge proof
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` if verification completes successfully (regardless of
    /// proof validity), or an error if the bb command fails.
    fn verify(
        &self,
        root: &Path,
        proof: &Path,
        public_input: &Path,
        vk: &Path,
        zk: bool,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

impl Default for Bb {
    fn default() -> Self {
        Self {
            system: Box::new(System),
        }
    }
}

impl TBb for Bb {
    /// Implements verification key generation using the bb CLI.
    /// 
    /// Executes `bb write_vk` with the Ultra Honk scheme and keccak oracle hash.
    /// The verification key is first written to `target/vk` and then copied to
    /// the specified output path if different.
    fn write_vk(
        &self,
        root: &Path,
        bytecode: &Path,
        output: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.system.execute_command(
            Command::new("bb")
                .arg("write_vk")
                .arg("--oracle_hash")
                .arg("keccak")
                .arg("-o")
                .arg("target")
                .arg("-b")
                .arg(bytecode)
                .current_dir(root),
        )?;

        if output != root.join("target").join("vk") {
            self.system.ensure_dir(output.parent().unwrap());
            self.system
                .copy_file(&root.join("target").join("vk"), output);
        }

        Ok(())
    }

    /// Implements proof generation using the bb CLI.
    /// 
    /// Executes `bb prove` with the Ultra Honk scheme and keccak oracle hash.
    /// Optionally includes the `--zk` flag for zero-knowledge proofs. The proof
    /// and public inputs are generated in the target directory and copied to
    /// the output path if different.
    fn prove(
        &self,
        root: &Path,
        bytecode: &Path,
        witness: &Path,
        output: &Path,
        zk: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // build command
        let mut command = Command::new("bb");
        command
            .arg("prove")
            .arg("-b")
            .arg(bytecode)
            .arg("-w")
            .arg(witness)
            .arg("-o")
            .arg(root.join("target"))
            .arg("--scheme")
            .arg("ultra_honk")
            .arg("--oracle_hash")
            .arg("keccak")
            .current_dir(root);

        // add zk flag if needed
        if zk {
            command.arg("--zk");
        }

        self.system.ensure_dir(&root.join("target"));
        self.system.execute_command(&mut command)?;

        // copy proof to output path if it's different
        if output != root.join("target") {
            self.system.ensure_dir(output.parent().unwrap());
            self.system
                .copy_file(&root.join("target").join("proof"), &output.join("proof"));
            self.system.copy_file(
                &root.join("target").join("public_inputs"),
                &output.join("public_inputs"),
            );
        }

        Ok(())
    }

    /// Implements local proof verification using the bb CLI.
    /// 
    /// Executes `bb verify` with the Ultra Honk scheme and keccak oracle hash.
    /// Optionally includes the `--zk` flag for zero-knowledge proof verification.
    /// The command will exit with a non-zero status if the proof is invalid.
    fn verify(
        &self,
        root: &Path,
        proof: &Path,
        public_input: &Path,
        vk: &Path,
        zk: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut command = Command::new("bb");
        command
            .arg("verify")
            .arg("--proof_path")
            .arg(proof)
            .arg("--public_inputs_path")
            .arg(public_input)
            .arg("--vk_path")
            .arg(vk)
            .arg("--oracle_hash")
            .arg("keccak")
            .arg("--scheme")
            .arg("ultra_honk")
            .current_dir(root);

        if zk {
            command.arg("--zk");
        }

        self.system.execute_command(&mut command)?;

        Ok(())
    }
}
