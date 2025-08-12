//! # CLI Commands
//!
//! This module contains all the command implementations for the NSV CLI.
//! Each command is responsible for a specific aspect of the Noir-to-Stylus workflow:
//!
//! - [`check`]: Validates Stylus contract compatibility and deployment costs
//! - [`deploy`]: Deploys generated verifier contracts to the blockchain
//! - [`generate`]: Creates Stylus verifier contracts from Noir circuits
//! - [`new`]: Creates new NSV projects with template files
//! - [`prove`]: Generates cryptographic proofs for circuits
//! - [`verify`]: Verifies proofs either locally or on-chain

pub(crate) mod check;
pub(crate) mod deploy;
pub(crate) mod generate;
pub(crate) mod new;
pub(crate) mod prove;
pub(crate) mod verify;
