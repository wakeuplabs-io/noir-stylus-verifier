use crate::infrastructure::system::{System, TSystem};
use std::{path::Path, process::Command};

pub(crate) struct Bb {
    system: Box<dyn TSystem>,
}

#[cfg_attr(test, mockall::automock)]
pub(crate) trait TBb {
    fn write_vk(
        &self,
        root: &Path,
        bytecode: &Path,
        output: &Path,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn prove(
        &self,
        root: &Path,
        bytecode: &Path,
        witness: &Path,
        output: &Path,
        zk: bool,
    ) -> Result<(), Box<dyn std::error::Error>>;
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

        self.system.execute_command(&mut command)?;

        // copy proof to output path if it's different
        if output != root.join("target").join("proof") {
            self.system.ensure_dir(output.parent().unwrap());
            self.system
                .copy_file(&root.join("target").join("proof"), output);
        }

        Ok(())
    }

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
