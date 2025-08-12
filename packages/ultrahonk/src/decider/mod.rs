//! # Decider Phase
//!
//! The decider phase is the second stage of Ultra Honk verification, responsible for
//! verifying the polynomial constraints through sumcheck and polynomial commitment
//! scheme verification. It consists of two main components:
//!
//! ## Sumcheck Protocol
//!
//! The [`sumcheck`] module implements the interactive sumcheck protocol verification,
//! which reduces the verification of polynomial constraints to polynomial evaluations
//! at a random point. This includes:
//! - Verifying individual relation constraints
//! - Managing the sumcheck rounds and challenges
//! - Computing final claimed evaluations
//!
//! ## Shplemini (Polynomial Commitment Scheme)
//!
//! The [`shplemini`] module implements the polynomial commitment scheme verification,
//! which verifies that the prover's polynomial commitments are consistent with
//! the claimed evaluations from sumcheck. This includes:
//! - Gemini fold verification
//! - KZG-style opening proofs
//! - Pairing checks for commitment verification

pub(crate) mod barycentric;
pub mod shplemini;
pub mod sumcheck;
pub mod types;
pub(crate) mod univariate;
