//! # Application Constants
//!
//! This module defines various constants used throughout the NSV CLI, including
//! default RPC URLs, verifier contract addresses, chain IDs, and system
//! requirements for external tools.

use crate::infrastructure::requirements::{Comparison, Requirement};

/// Default RPC URL used when none is specified
pub const DEFAULT_RPC_URL: &str = "https://sepolia-rollup.arbitrum.io/rpc";

/// Global Verifier contract address for Arbitrum mainnet (standard)
pub const VERIFIER_ADDRESS_ARBITRUM: &str = "0x2f9f4741ab606632718f7bda0bf5c79e1dd03ac3";

///  Global Verifier contract address for Arbitrum Sepolia testnet (standard)
pub const VERIFIER_ADDRESS_ARBITRUM_SEPOLIA: &str = "0x951d400a88f98c2d3f6f8af7b502a59bf418ab76";

/// Global  Verifier contract address for Arbitrum mainnet (zero-knowledge)
pub const VERIFIER_ADDRESS_ARBITRUM_ZK: &str = "0x79693edb49473dc3522de16fbd047977c4999d5c";

/// Global Verifier contract address for Arbitrum Sepolia testnet (zero-knowledge)
pub const VERIFIER_ADDRESS_ARBITRUM_SEPOLIA_ZK: &str = "0xdcaaed24c926bc718984eaa4126e27b27d60379d";

/// Chain ID for Arbitrum mainnet
pub const CHAIN_ID_ARBITRUM: u64 = 42161;

/// Chain ID for Arbitrum Sepolia testnet
pub const CHAIN_ID_ARBITRUM_SEPOLIA: u64 = 421614;

/// System requirement for cargo-stylus CLI tool
/// 
/// cargo-stylus is required for checking and deploying Stylus contracts.
/// Version 0.6.0 or higher is required.
pub(crate) const CARGO_STYLUS_REQUIREMENT: Requirement = Requirement {
    program: "cargo-stylus",
    version_arg: "--version",
    required_version: "0.6.0",
    required_comparator: Comparison::GreaterThanOrEqual,
    required_hash: &[],
};

/// System requirement for Barretenberg (bb) CLI tool
/// 
/// bb is the Barretenberg backend used for writing vk, proving and verifying locally.
/// Exactly version 0.86.0 is required. Since bb doesn't have a reliable version
/// command, we also verify by binary hash.
pub(crate) const BB_REQUIREMENT: Requirement = Requirement {
    program: "bb",
    version_arg: "--version",
    required_version: "0.86.0",
    required_comparator: Comparison::Equal,
    required_hash: &[
        // bb doesn't have a version command so we use the hash of the binary
        "0caa9112cd5e446ea336ef9476f0532366dd856f0b2c4ffbd0abd32635c10052", // amd64-darwin
        "f09a13bfba9797d9569da5a45380354176bdf4ada6409f710640a21ddd06ba40", // arm64-darwin
        "6a73c1d9e72ecc29c569b82012173837c1acb00c6759efe9b995ee0b2ee29c82", // arm64-linux
        "9491a70fc9f37381760e36240b5e67e6f9baeeca969bd2213c4f57f9349f6b66", // amd64-linux
    ],
};

/// System requirement for Nargo CLI tool
/// 
/// Nargo is the Noir language toolchain used for compiling circuits and
/// executing them to generate witnesses. Exactly version 1.0.0-beta.6 is required.
pub(crate) const NARGO_REQUIREMENT: Requirement = Requirement {
    program: "nargo",
    version_arg: "--version",
    required_version: "1.0.0-beta.6",
    required_comparator: Comparison::Equal,
    required_hash: &[],
};
