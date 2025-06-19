use crate::types::ScalarField;
use alloc::vec::Vec;
pub(crate) mod round_verifier;
pub(crate) mod verifier;

pub struct SumcheckVerifierOutput {
    pub multivariate_challenge: Vec<ScalarField>,
    pub verified: bool,
}
