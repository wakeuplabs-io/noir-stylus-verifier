use std::{path::Path, process::Command};

use crate::infrastructure::system::{System, TSystem};

pub(crate) struct Stylus {
    system: System,
}

impl Stylus {
    pub(crate) fn new() -> Self {
        Self { system: System::new() }
    }

    pub(crate) fn build(&self, root: &Path) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub(crate) fn deploy(&self, root: &Path) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub(crate) fn check(&self, root: &Path, rpc_url: &str) -> Result<String, Box<dyn std::error::Error>> {
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

