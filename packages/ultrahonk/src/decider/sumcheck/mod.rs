use crate::types::ScalarField;
use alloc::vec::Vec;
pub(crate) mod relations;
pub(crate) mod round_verifier;
pub mod verifier;

pub struct SumcheckVerifierOutput {
    pub multivariate_challenge: Vec<ScalarField>,
    pub verified: bool,
}
