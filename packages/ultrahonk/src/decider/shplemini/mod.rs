use alloc::vec::Vec;
pub mod types;
pub mod verifier;
use crate::types::{G1Affine, ScalarField};

pub struct ShpleminiVerifierOpeningClaim {
    pub challenge: ScalarField,
    pub scalars: Vec<ScalarField>,
    pub commitments: Vec<G1Affine>,
}
