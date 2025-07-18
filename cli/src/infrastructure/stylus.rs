use std::{path::Path, process::Command};

use crate::infrastructure::system::{System, TSystem};

pub(crate) struct Stylus {
    system: System,
}

impl Stylus {
    pub(crate) fn new() -> Self {
        Self { system: System::new() }
    }

    pub(crate) fn deploy(&self, root: &Path, rpc_url: &str, private_key: &str, constructor_args: &str) -> Result<String, Box<dyn std::error::Error>> {
        let result = self.system.execute_command(
            Command::new("cargo")
                .arg("stylus")
                .arg("deploy")
                .arg("--endpoint")
                .arg(rpc_url)
                .arg("--private-key")
                .arg(private_key)
                .arg("--constructor-args")
                .arg(constructor_args)
                .arg("--no-verify")
                .current_dir(root),
        )?;

        Ok(result)
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

