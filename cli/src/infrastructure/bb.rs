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
        circuit_path: &Path,
        package_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

impl Default for Bb {
    fn default() -> Self {
        Self {
            system: Box::new(System::default()),
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
        circuit_path: &Path,
        package_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.system.execute_command(
            Command::new("bb")
                .arg("write_vk")
                .arg("--oracle_hash")
                .arg("keccak")
                .arg("-o")
                .arg("target")
                .arg("-b")
                .arg(
                    circuit_path
                        .join("target")
                        .join(format!("{package_name}.json")),
                )
                .current_dir(&circuit_path),
        )?;
        Ok(())
    }
}
