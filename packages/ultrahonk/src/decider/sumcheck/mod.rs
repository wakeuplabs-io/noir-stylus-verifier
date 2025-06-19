use alloc::vec::Vec;
pub(crate) mod round_verifier;
pub(crate) mod verifier;
use ark_ff::PrimeField;

pub struct SumcheckVerifierOutput<F: PrimeField> {
    pub multivariate_challenge: Vec<F>,
    pub verified: bool,
}
