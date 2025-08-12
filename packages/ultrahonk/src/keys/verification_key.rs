//! # Verification Key Structure
//!
//! This module defines the verification key structure used in Ultra Honk
//! proof verification. The verification key contains all circuit-specific
//! parameters and polynomial commitments needed to verify proofs.

use crate::types::{G1Affine, PrecomputedEntities};

/// Ultra Honk verification key containing circuit-specific parameters.
///
/// The verification key contains all the information needed to verify proofs
/// for a specific circuit, including the circuit size, polynomial commitments
/// for precomputed entities, and public input configuration.
pub struct VerifyingKey {
    /// The size of the circuit (number of rows in the execution trace)
    pub circuit_size: u32,
    /// Commitments to precomputed polynomials (selectors, permutations, tables, etc.)
    pub commitments: PrecomputedEntities<G1Affine>,
    /// The number of public inputs expected by this circuit
    pub num_public_inputs: u32,
    /// Offset where public inputs begin in the witness polynomial
    pub pub_inputs_offset: u32,
}
