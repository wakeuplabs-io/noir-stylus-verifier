//! # System Operations
//!
//! This module provides a system abstraction layer for file operations, process
//! execution, and other OS-level interactions. It enables testing by abstracting
//! system calls behind a trait interface while providing concrete implementations
//! for production use.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// System operations implementation for production use.
///
/// This struct provides concrete implementations of all system operations
/// using standard library functions for file I/O, process execution, and
/// other OS-level interactions.
#[derive(Default)]
pub(crate) struct System;

/// Trait defining the interface for system operations.
///
/// This trait abstracts all system-level operations to enable testing and
/// provide a clean interface for file operations, process execution, and
/// other OS interactions.
#[cfg_attr(test, mockall::automock)]
pub(crate) trait TSystem: Send + Sync {
    /// Executes a command and returns its output.
    ///
    /// Runs the specified command and captures both stdout and stderr.
    /// Returns the stdout as a string on success, or an error message
    /// containing stderr on failure.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute with its arguments
    ///
    /// # Returns
    ///
    /// Returns the command's stdout on success, or an error string on failure.
    fn execute_command(&self, command: &mut Command) -> Result<String, String>;

    /// Writes content to a file, creating directories as needed.
    ///
    /// Creates the parent directory structure if it doesn't exist,
    /// then writes the content to the specified file path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the file should be written
    /// * `content` - Content to write to the file
    fn write_file(&self, path: &Path, content: String);

    /// Reads a file's content as a UTF-8 string.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to read
    ///
    /// # Returns
    ///
    /// Returns the file content as a string.
    fn read_file_str(&self, path: &Path) -> String;

    /// Reads a file's content as raw bytes.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to read
    ///
    /// # Returns
    ///
    /// Returns the file content as a byte vector.
    fn read_file(&self, path: &Path) -> Vec<u8>;

    /// Creates a directory and all parent directories as needed.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the directory to create
    fn ensure_dir(&self, path: &Path);

    /// Checks if a file or directory exists.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to check for existence
    ///
    /// # Returns
    ///
    /// Returns true if the path exists, false otherwise.
    fn exists(&self, path: &Path) -> bool;

    /// Returns the current working directory.
    ///
    /// # Returns
    ///
    /// Returns the current working directory as a PathBuf.
    fn current_dir(&self) -> PathBuf;

    /// Copies a file from source to destination.
    ///
    /// Creates the destination directory structure if needed,
    /// then copies the file contents.
    ///
    /// # Arguments
    ///
    /// * `source` - Path to the source file
    /// * `destination` - Path where the file should be copied
    fn copy_file(&self, source: &Path, destination: &Path);

    /// Locates an executable in the system PATH.
    ///
    /// Uses the `which` command to find the full path to an executable.
    ///
    /// # Arguments
    ///
    /// * `command` - Name of the command to locate
    ///
    /// # Returns
    ///
    /// Returns the full path to the executable if found, None otherwise.
    fn which(&self, command: &str) -> Option<PathBuf>;
}

// implementations ==========================================

impl TSystem for System {
    /// Implements command execution using the standard library.
    ///
    /// Executes the command and captures output, returning stdout on success
    /// or a formatted error message including stderr on failure.
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

    /// Implements file writing with automatic directory creation.
    fn write_file(&self, path: &Path, content: String) {
        let parent_dir = path.parent().unwrap();
        std::fs::create_dir_all(parent_dir).unwrap();
        std::fs::write(path, content).unwrap();
    }

    /// Implements binary file reading.
    fn read_file(&self, path: &Path) -> Vec<u8> {
        std::fs::read(path).unwrap()
    }

    /// Implements UTF-8 file reading.
    fn read_file_str(&self, path: &Path) -> String {
        std::fs::read_to_string(path).unwrap()
    }

    /// Implements current directory retrieval.
    fn current_dir(&self) -> PathBuf {
        std::env::current_dir().unwrap()
    }

    /// Implements directory creation with parent directories.
    fn ensure_dir(&self, path: &Path) {
        std::fs::create_dir_all(path).unwrap();
    }

    /// Implements path existence checking.
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    /// Implements file copying with directory creation.
    fn copy_file(&self, source: &Path, destination: &Path) {
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::copy(source, destination).unwrap();
    }

    /// Implements executable location using the which command.
    fn which(&self, command: &str) -> Option<PathBuf> {
        let output = self
            .execute_command(Command::new("which").arg(command))
            .map_err(|_| Option::<PathBuf>::None)
            .ok()?;
        Some(PathBuf::from(output.trim()))
    }
}
