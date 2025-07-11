use crate::types::ScalarField;
use alloc::vec::Vec;
pub mod relations;
pub mod round_verifier;
pub mod verifier;

pub struct SumcheckVerifierOutput {
    pub multivariate_challenge: Vec<ScalarField>,
    pub verified: bool,
}
