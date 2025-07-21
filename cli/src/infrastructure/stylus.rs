use std::{path::Path, process::Command};

use crate::infrastructure::system::{System, TSystem};

pub(crate) struct Stylus {
    system: System,
}

#[cfg_attr(test, mockall::automock)]
pub(crate) trait TStylus: Send + Sync {
    fn deploy(
        &self,
        root: &Path,
        rpc_url: &str,
        private_key: &str,
        constructor_args: &str,
    ) -> Result<String, Box<dyn std::error::Error>>;
    fn check(&self, root: &Path, rpc_url: &str) -> Result<String, Box<dyn std::error::Error>>;
}

// implementations ==========================================

impl Stylus {
    pub(crate) fn new() -> Self {
        Self {
            system: System::new(),
        }
    }
}

impl TStylus for Stylus {
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
