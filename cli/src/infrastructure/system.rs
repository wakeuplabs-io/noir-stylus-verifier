use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Default)]
pub(crate) struct System;

#[cfg_attr(test, mockall::automock)]
pub(crate) trait TSystem: Send + Sync {
    fn execute_command(&self, command: &mut Command) -> Result<String, String>;
    fn write_file(&self, path: &Path, content: String);
    fn read_file_str(&self, path: &Path) -> String;
    fn read_file(&self, path: &Path) -> Vec<u8>;
    fn ensure_dir(&self, path: &Path);
    fn exists(&self, path: &Path) -> bool;
    fn current_dir(&self) -> PathBuf;
    fn copy_file(&self, source: &Path, destination: &Path);
}

// implementations ==========================================

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

    fn write_file(&self, path: &Path, content: String) {
        let parent_dir = path.parent().unwrap();
        std::fs::create_dir_all(parent_dir).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn read_file(&self, path: &Path) -> Vec<u8> {
        std::fs::read(path).unwrap()
    }

    fn read_file_str(&self, path: &Path) -> String {
        std::fs::read_to_string(path).unwrap()
    }

    fn current_dir(&self) -> PathBuf {
        std::env::current_dir().unwrap()
    }

    fn ensure_dir(&self, path: &Path) {
        std::fs::create_dir_all(path).unwrap();
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn copy_file(&self, source: &Path, destination: &Path) {
        println!(
            "Copying file from {} to {}",
            source.display(),
            destination.display()
        );
        // ensure destination directory exists
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::copy(source, destination).unwrap();
    }
}
