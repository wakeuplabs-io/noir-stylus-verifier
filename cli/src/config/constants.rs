use crate::infrastructure::requirements::{Comparison, Requirement};

pub const DEFAULT_RPC_URL: &str = "https://sepolia-rollup.arbitrum.io/rpc";
pub const VERIFIER_ADDRESS_ARBITRUM: &str = "0x0000000000000000000000000000000000000000";
pub const VERIFIER_ADDRESS_ARBITRUM_SEPOLIA: &str = "0x951d400a88f98c2d3f6f8af7b502a59bf418ab76";
pub const VERIFIER_ADDRESS_ARBITRUM_ZK: &str = "0x0000000000000000000000000000000000000000";
pub const VERIFIER_ADDRESS_ARBITRUM_SEPOLIA_ZK: &str = "0xdcaaed24c926bc718984eaa4126e27b27d60379d";
pub const CHAIN_ID_ARBITRUM: u64 = 42161;
pub const CHAIN_ID_ARBITRUM_SEPOLIA: u64 = 421614;

pub(crate) const CARGO_STYLUS_REQUIREMENT: Requirement = Requirement {
    program: "cargo-stylus",
    version_arg: "--version",
    required_version: "0.1.0",
    required_comparator: Comparison::GreaterThanOrEqual,
    required_hash: &[],
};

pub(crate) const BB_REQUIREMENT: Requirement = Requirement {
    program: "bb",
    version_arg: "--version",
    required_version: "0.86.0",
    required_comparator: Comparison::Equal,
    required_hash: &[
        "0caa9112cd5e446ea336ef9476f0532366dd856f0b2c4ffbd0abd32635c10052", // amd64-darwin
        "f09a13bfba9797d9569da5a45380354176bdf4ada6409f710640a21ddd06ba40", // arm64-darwin
        "6a73c1d9e72ecc29c569b82012173837c1acb00c6759efe9b995ee0b2ee29c82", // arm64-linux
        "9491a70fc9f37381760e36240b5e67e6f9baeeca969bd2213c4f57f9349f6b66", // amd64-linux
    ],
};

pub(crate) const NARGO_REQUIREMENT: Requirement = Requirement {
    program: "nargo",
    version_arg: "--version",
    required_version: "1.0.0-beta.6",
    required_comparator: Comparison::Equal,
    required_hash: &[],
};
