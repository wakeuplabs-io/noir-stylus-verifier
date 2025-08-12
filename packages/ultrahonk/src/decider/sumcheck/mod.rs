//! # Sumcheck Protocol Verification
//!
//! This module implements the sumcheck protocol verification for Ultra Honk.
//! The sumcheck protocol allows the verifier to check that a multivariate polynomial
//! sums to a claimed value over a hypercube, by reducing it to univariate polynomial
//! evaluations through multiple rounds of interaction.
//!
//! ## Components
//!
//! - [`verifier`]: Main sumcheck verifier logic and round management
//! - [`relations`]: Individual constraint relations that define the circuit
//! - [`round_verifier`]: Per-round verification logic and state management

use crate::types::ScalarField;
use alloc::vec::Vec;
pub(crate) mod relations;
pub(crate) mod round_verifier;
pub mod verifier;

/// Output from the sumcheck verification process.
///
/// This structure contains the results of sumcheck verification, including
/// the multivariate challenge point and the verification status.
pub struct SumcheckVerifierOutput {
    /// The random challenge point where polynomials should be evaluated
    pub multivariate_challenge: Vec<ScalarField>,
    /// Whether the sumcheck verification passed
    pub verified: bool,
    /// The claimed evaluation of the Libra polynomial (used in ZK flavors)
    pub claimed_libra_evaluation: ScalarField,
}
