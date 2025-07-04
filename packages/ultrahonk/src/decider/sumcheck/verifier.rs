use super::SumcheckVerifierOutput;
use crate::alloc::{borrow::ToOwned, string::ToString};
use crate::backends::G1ArithmeticBackend;
use crate::{
    backends::HashBackend,
    decider::{
        sumcheck::round_verifier::{SumcheckRoundOutput, SumcheckVerifierRound},
        types::GateSeparatorPolynomial,
        verifier::DeciderVerifier,
    },
    transcript::Transcript,
    types::{ScalarField, NUM_ALL_ENTITIES},
    verifier::HonkVerifyResult,
    CONST_PROOF_SIZE_LOG_N,
};
use alloc::vec::Vec;
use ark_ff::{One, Zero};
use crate::decider::types::BATCHED_RELATION_PARTIAL_LENGTH;


// Keep in mind, the UltraHonk protocol (UltraFlavor) does not per default have ZK
impl<P: G1ArithmeticBackend, H: HashBackend> DeciderVerifier<P, H> {
    pub fn verify_sumcheck(
        &mut self,
        transcript: &mut Transcript,
        circuit_size: u32,
    ) -> HonkVerifyResult<SumcheckVerifierOutput> {
        let log_circuit_size = circuit_size.ilog2();

        // determine padding indicator array
        let mut padding_indicator_array = [ScalarField::zero(); CONST_PROOF_SIZE_LOG_N];
        for (idx, value) in padding_indicator_array.iter_mut().enumerate() {
            *value = if idx < log_circuit_size as usize {
                ScalarField::one()
            } else {
                ScalarField::zero()
            };
        }

        let mut verified: bool = true;

        // Pad gate challenges for Protogalaxy DeciderVerifier and AVM
        if self.memory.relation_parameters.gate_challenges.len() < CONST_PROOF_SIZE_LOG_N {
            let zero = ScalarField::zero();
            for _ in self.memory.relation_parameters.gate_challenges.len()..CONST_PROOF_SIZE_LOG_N {
                self.memory.relation_parameters.gate_challenges.push(zero);
            }
        }

        let mut gate_separators = GateSeparatorPolynomial::new_without_products(
            self.memory.relation_parameters.gate_challenges.to_owned(),
        );

        let mut sum_check_round = SumcheckVerifierRound::default();
        let mut multivariate_challenge = Vec::with_capacity(CONST_PROOF_SIZE_LOG_N);

        for (round_idx, &padding_value) in padding_indicator_array.iter().enumerate() {
            let round_univariate_label = format!("Sumcheck:univariate_{}", round_idx);

            let evaluations =
                transcript.receive_fr_array_from_verifier::<BATCHED_RELATION_PARTIAL_LENGTH>(round_univariate_label)?;
            let round_univariate = SumcheckRoundOutput { evaluations };

            let round_challenge = transcript.get_challenge::<H>(format!("Sumcheck:u_{}", round_idx));

            let checked = sum_check_round.check_sum(&round_univariate, padding_value);
            verified = verified && checked; // TODO: this gets overwritten by the final round

            multivariate_challenge.push(round_challenge);

            sum_check_round.compute_next_target_sum(
                &round_univariate,
                round_challenge,
                padding_value,
            );
            gate_separators.partially_evaluate_with_padding(
                round_challenge,
                padding_indicator_array[round_idx],
            );
        }

        // Final round
        let transcript_evaluations = transcript
            .receive_fr_vec_from_verifier("Sumcheck:evaluations".to_string(), NUM_ALL_ENTITIES)?;

        for (eval, &transcript_eval) in self
            .memory
            .claimed_evaluations
            .iter_mut()
            .zip(transcript_evaluations.iter())
        {
            *eval = transcript_eval;
        }

        // Evaluate the Honk relation at the point (u_0, ..., u_{d-1}) using claimed evaluations of prover polynomials.
        let full_honk_purported_value =
            SumcheckVerifierRound::compute_full_relation_purported_value(
                &self.memory.claimed_evaluations,
                &self.memory.relation_parameters,
                gate_separators,
            );

        let checked = full_honk_purported_value == sum_check_round.target_total_sum;
        verified = verified && checked;
        Ok(SumcheckVerifierOutput {
            multivariate_challenge,
            verified,
        })
    }
}
