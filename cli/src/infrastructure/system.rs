use std::{path::Path, process::Command};

pub(crate) struct System;

#[cfg_attr(test, mockall::automock)]
pub(crate) trait TSystem: Send + Sync {
    fn execute_command(&self, command: &mut Command) -> Result<String, String>;
    fn write_file(&self, path: &Path, content: String) -> Result<(), String>;
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, String>;
    fn read_file_str(&self, path: &Path) -> Result<String, String>;
    fn ensure_dir(&self, path: &Path) -> Result<(), String>;
    fn exists(&self, path: &Path) -> bool;
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

    fn write_file(&self, path: &Path, content: String) -> Result<(), String> {
        let parent_dir = path.parent().unwrap();
        std::fs::create_dir_all(parent_dir)
            .map_err(|e| format!("Failed to create directory: {e}"))?;
        std::fs::write(path, content).map_err(|e| format!("Failed to write file: {e}"))
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, String> {
        std::fs::read(path).map_err(|e| format!("Failed to read file: {e}"))
    }

    fn read_file_str(&self, path: &Path) -> Result<String, String> {
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {e}"))
    }

    fn ensure_dir(&self, path: &Path) -> Result<(), String> {
        std::fs::create_dir_all(path).map_err(|e| format!("Failed to create directory: {e}"))
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}
