//! # Infrastructure
//!
//! This module contains the infrastructure layer of the NSV CLI, providing
//! abstractions and implementations for interacting with external tools,
//! the file system, and various services. Each submodule handles a specific
//! aspect of the system's infrastructure:
//!
//! - [`bb`]: Barretenberg backend integration for cryptographic operations
//! - [`codegen`]: Code generation for Stylus contracts and project templates
//! - [`nargo`]: Nargo (Noir toolchain) integration
//! - [`progress`]: Progress indicators and user feedback
//! - [`requirements`]: System dependency checking and validation
//! - [`rpc`]: RPC client for blockchain interactions
//! - [`stylus`]: Stylus CLI integration for contract operations
//! - [`system`]: File system and process operations
//! - [`templates`]: Template files for code generation
//! - [`terminal`]: Terminal UI utilities and formatting
//! - [`utils`]: General utility functions

pub(crate) mod bb;
pub(crate) mod codegen;
pub(crate) mod nargo;
pub(crate) mod progress;
pub(crate) mod requirements;
pub(crate) mod rpc;
pub(crate) mod stylus;
pub(crate) mod system;
pub(crate) mod templates;
pub(crate) mod terminal;
pub(crate) mod utils;
