//! # Utility Functions
//!
//! This module provides utility functions for common operations used throughout
//! the CLI, including cryptographic hashing for binary verification and other
//! shared functionality.

use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

/// Trait defining the interface for SHA256 hashing operations.
/// 
/// This trait abstracts SHA256 hashing functionality to enable testing
/// and different implementation strategies for file hashing.
#[cfg_attr(test, mockall::automock)]
pub(crate) trait TSha256Hasher: Send + Sync {
    /// Computes the SHA256 hash of a file.
    /// 
    /// Reads the file at the specified path and computes its SHA256 hash,
    /// returning the hash as a hexadecimal string.
    /// 
    /// # Arguments
    /// 
    /// * `path` - Path to the file to hash
    /// 
    /// # Returns
    /// 
    /// Returns the SHA256 hash as a lowercase hexadecimal string,
    /// or an I/O error if the file cannot be read.
    fn hash(&self, path: &Path) -> std::io::Result<String>;
}

/// SHA256 hasher implementation using the sha2 crate.
/// 
/// This struct provides SHA256 hashing functionality for file verification,
/// particularly used in system requirements checking where binary hashes
/// are compared for tool verification.
#[derive(Default)]
pub(crate) struct Sha256Hasher;

impl TSha256Hasher for Sha256Hasher {
    /// Implements SHA256 hashing by reading the file in chunks.
    /// 
    /// Opens the file, reads it in 8KB chunks to handle large files efficiently,
    /// and computes the SHA256 hash incrementally. Returns the hash as a
    /// lowercase hexadecimal string.
    fn hash(&self, path: &Path) -> std::io::Result<String> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        Ok(hex::encode(hasher.finalize()))
    }
}
