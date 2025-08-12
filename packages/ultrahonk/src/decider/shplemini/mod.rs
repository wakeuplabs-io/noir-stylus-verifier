//! # Shplemini Polynomial Commitment Scheme
//!
//! This module implements the Shplemini polynomial commitment scheme verification
//! for Ultra Honk. Shplemini is a KZG-based polynomial commitment scheme that
//! provides efficient verification of polynomial evaluations.
//!
//! ## Components
//!
//! - [`verifier`]: Main Shplemini verification logic
//! - [`types`]: Data structures specific to Shplemini verification
//!
//! The verification process involves:
//! 1. Gemini fold verification to batch multiple opening claims
//! 2. KZG verification of the batched opening proof
//! 3. Pairing checks to verify commitment consistency

use alloc::vec::Vec;
pub mod types;
pub mod verifier;
use crate::types::{G1Affine, ScalarField};

/// A polynomial opening claim for Shplemini verification.
/// 
/// This structure represents a claim that a set of polynomials evaluate to
/// specific values at a challenge point, backed by their commitments.
pub(crate) struct ShpleminiVerifierOpeningClaim {
    /// The challenge point where polynomials are claimed to evaluate
    pub(crate) challenge: ScalarField,
    /// The claimed evaluation values for each polynomial
    pub(crate) scalars: Vec<ScalarField>,
    /// The polynomial commitments being verified
    pub(crate) commitments: Vec<G1Affine>,
}
