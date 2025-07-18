use std::process::Command;

pub(crate) struct System;

#[cfg_attr(test, mockall::automock)]
pub(crate) trait TSystem: Send + Sync {
    fn execute_command(&self, command: &mut Command) -> Result<String, String>;
}

// implementations ==========================================

impl System {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl TSystem for System {
    fn execute_command(&self, command: &mut Command) -> Result<String, String> {
        let output = command
            .output()
            .map_err(|e| format!("Failed to execute command: {e}"))?;

        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(result)
        } else {
            let error_message = String::from_utf8_lossy(&output.stderr).to_string();
            Err(format!("Command failed with error: {error_message}"))
        }
    }
}
