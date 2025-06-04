pub(crate) mod types;
pub(crate) mod verifier;

use ark_ec::pairing::Pairing;

pub(crate) struct ShpleminiVerifierOpeningClaim<P: Pairing> {
    pub(crate) challenge: P::ScalarField,
    pub(crate) scalars: Vec<P::ScalarField>,
    pub(crate) commitments: Vec<P::G1Affine>,
}
