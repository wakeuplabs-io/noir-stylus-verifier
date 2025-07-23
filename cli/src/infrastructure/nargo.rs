use crate::infrastructure::system::{System, TSystem};
use std::{path::Path, process::Command};

#[cfg_attr(test, mockall::automock)]
pub(crate) trait TNargo {
    fn setup(&self, version: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn compile(&self, circuit_path: &Path) -> Result<(), Box<dyn std::error::Error>>;
}

pub(crate) struct Nargo {
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
    fn setup(&self, version: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.system
            .execute_command(Command::new("noirup").arg("-v").arg(version))?;

        Ok(())
    }

    fn compile(&self, circuit_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        self.system.execute_command(
            Command::new("nargo")
                .arg("compile")
                .current_dir(circuit_path),
        )?;

        Ok(())
    }
}
