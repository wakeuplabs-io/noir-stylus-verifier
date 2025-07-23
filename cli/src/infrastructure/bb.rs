use std::{path::Path, process::Command};

use crate::infrastructure::system::{System, TSystem};

pub(crate) struct Bb {
    system: Box<dyn TSystem>,
}

#[cfg_attr(test, mockall::automock)]
pub(crate) trait TBb {
    fn setup(&self, version: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn write_vk(
        &self,
        root: &Path,
        package_name: &str,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn prove(
        &self,
        root: &Path,
        package_name: &str,
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
    fn setup(&self, version: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.system
            .execute_command(Command::new("bbup").arg("-v").arg(version))?;
        Ok(())
    }

    fn write_vk(
        &self,
        root: &Path,
        package_name: &str,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bytecode_path = root.join("target").join(format!("{package_name}.json"));
        self.system.execute_command(
            Command::new("bb")
                .arg("write_vk")
                .arg("--oracle_hash")
                .arg("keccak")
                .arg("-o")
                .arg("target")
                .arg("-b")
                .arg(bytecode_path)
                .current_dir(root),
        )?;

        self.system.ensure_dir(output_path.parent().unwrap());
        self.system
            .copy_file(&root.join("target").join("vk"), output_path);

        Ok(())
    }

    fn prove(
        &self,
        root: &Path,
        package_name: &str,
        zk: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bytecode_path = root.join("target").join(format!("{package_name}.json"));
        let witness_path = root.join("target").join(format!("{package_name}.gz"));

        // build command
        let mut command = Command::new("bb");
        command
            .arg("prove")
            .arg("-b")
            .arg(&bytecode_path)
            .arg("-w")
            .arg(&witness_path)
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
