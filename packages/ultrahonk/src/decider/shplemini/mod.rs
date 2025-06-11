pub(crate) mod types;
pub(crate) mod verifier;
use crate::types::{G1Affine, ScalarField};

pub(crate) struct ShpleminiVerifierOpeningClaim {
    pub(crate) challenge: ScalarField,
    pub(crate) scalars: Vec<ScalarField>,
    pub(crate) commitments: Vec<G1Affine>,
}
