use alloc::vec::Vec;
pub mod types;
pub mod verifier;
use crate::types::{G1Affine, ScalarField};

pub(crate) struct ShpleminiVerifierOpeningClaim {
    pub(crate) challenge: ScalarField,
    pub(crate) scalars: Vec<ScalarField>,
    pub(crate) commitments: Vec<G1Affine>,
}
