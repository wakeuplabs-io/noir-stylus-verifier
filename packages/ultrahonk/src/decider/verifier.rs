use super::{shplemini::ShpleminiVerifierOpeningClaim, types::VerifierMemory};
use crate::alloc::string::ToString;
use crate::backends::G1ArithmeticBackend;
use crate::types::HonkProofError;
use crate::{
    backends::HashBackend,
    decider::types::BATCHED_RELATION_PARTIAL_LENGTH,
    transcript::Transcript,
    types::{G1Affine, G2Affine, ScalarField},
    verifier::HonkVerifyResult,
    Utils, CONST_PROOF_SIZE_LOG_N,
};
use ark_ec::AffineRepr;
use ark_ff::{One, Zero};
use core::marker::PhantomData;

pub(crate) struct DeciderVerifier<P: G1ArithmeticBackend, H: HashBackend> {
    pub(super) memory: VerifierMemory,
    phantom_data: PhantomData<P>,
    phantom_hasher: PhantomData<H>,
}

impl<P: G1ArithmeticBackend, H: HashBackend> DeciderVerifier<P, H> {
    pub(crate) fn new(memory: VerifierMemory) -> Self {
        Self {
            memory,
            phantom_data: PhantomData,
            phantom_hasher: PhantomData,
        }
    }

    pub(crate) fn reduce_verify_shplemini(
        opening_pair: &mut ShpleminiVerifierOpeningClaim,
        mut transcript: Transcript<H>,
    ) -> HonkVerifyResult<(G1Affine, G1Affine)> {
        let quotient_commitment = transcript.receive_point_from_prover("KZG:W".to_string())?;
        opening_pair.commitments.push(quotient_commitment);
        opening_pair.scalars.push(opening_pair.challenge);
        let p_1 = -quotient_commitment.into_group();

        let p_0 = P::msm(&opening_pair.scalars, &opening_pair.commitments)
            .map_err(|_| HonkProofError::MSMError)?;

        Ok((p_0, p_1.into()))
    }

    pub(crate) fn verify(
        mut self,
        circuit_size: u32,
        crs: &G2Affine,
        mut transcript: Transcript<H>,
    ) -> HonkVerifyResult<bool> {
        let log_circuit_size = Utils::get_msb32(circuit_size);

        let mut padding_indicator_array = [ScalarField::zero(); CONST_PROOF_SIZE_LOG_N];

        for (idx, value) in padding_indicator_array.iter_mut().enumerate() {
            *value = if idx < log_circuit_size as usize {
                ScalarField::one()
            } else {
                ScalarField::zero()
            };
        }
        let sumcheck_output = {
            let sumcheck_output = self.sumcheck_verify::<BATCHED_RELATION_PARTIAL_LENGTH>(
                &mut transcript,
                &padding_indicator_array,
            )?;
            if !sumcheck_output.verified {
                return Ok(false);
            }

            sumcheck_output
        };

        let mut opening_claim = self.compute_batch_opening_claim(
            sumcheck_output.multivariate_challenge,
            &mut transcript,
            &padding_indicator_array,
        )?;

        let pairing_points = Self::reduce_verify_shplemini(&mut opening_claim, transcript)?;
        let pcs_verified = P::ec_pairing_check(
            pairing_points.0,
            pairing_points.1,
            *crs,
            G2Affine::generator(),
        )
        .unwrap();
        Ok(sumcheck_output.verified && pcs_verified)
    }
}
