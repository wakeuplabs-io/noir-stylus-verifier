use crate::types::ScalarField;
use alloc::vec::Vec;
pub mod round_verifier;
pub mod verifier;
pub mod relations;

pub struct SumcheckVerifierOutput {
    pub multivariate_challenge: Vec<ScalarField>,
    pub verified: bool,
}
