use super::SumcheckVerifierOutput;
use crate::alloc::borrow::ToOwned;
use crate::constants::NUM_LIBRA_COMMITMENTS;
use crate::decider::types::{ClaimedEvaluations, RelationParameters};
use crate::oink::types::Challenges;
use crate::polynomials::RowDisablingPolynomial;
use crate::types::G1Affine;
use crate::{
    backends::HashBackend,
    decider::{
        sumcheck::round_verifier::{SumcheckRoundOutput, SumcheckVerifierRound},
        types::GateSeparatorPolynomial,
    },
    transcript::Transcript,
    types::{HonkVerifyResult, ScalarField, NUM_ALL_ENTITIES},
    CONST_PROOF_SIZE_LOG_N,
};
use alloc::vec::Vec;
use ark_ff::{One, Zero};

pub struct SumcheckVerifierMemory {
    pub relation_parameters: RelationParameters,
    pub claimed_evaluations: ClaimedEvaluations,
}

impl SumcheckVerifierMemory {
    pub fn new(
        relation_parameters: RelationParameters,
        claimed_evaluations: ClaimedEvaluations,
    ) -> Self {
        Self {
            relation_parameters,
            claimed_evaluations,
        }
    }

    pub fn from_memory_and_gate_challenges<H: HashBackend>(
        challenges: &Challenges,
        gate_challenges: Vec<ScalarField>,
        public_input_delta: ScalarField,
    ) -> Self {
        let relation_parameters = RelationParameters {
            eta_1: challenges.eta_1,
            eta_2: challenges.eta_2,
            eta_3: challenges.eta_3,
            beta: challenges.beta,
            gamma: challenges.gamma,
            public_input_delta,
            alphas: challenges.alphas,
            gate_challenges,
        };

        Self {
            relation_parameters,
            claimed_evaluations: Default::default(),
        }
    }
}

pub struct SumcheckVerifier {
    pub memory: SumcheckVerifierMemory,
}

impl SumcheckVerifier {
    pub fn new(memory: SumcheckVerifierMemory) -> Self {
        Self { memory }
    }

    pub fn verify_sumcheck<H: HashBackend, const SIZE: usize>(
        &mut self,
        transcript: &mut Transcript,
        circuit_size: u32,
        has_zk: bool,
    ) -> HonkVerifyResult<(SumcheckVerifierOutput, Vec<G1Affine>)> {
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

        let mut libra_commitments = Vec::with_capacity(NUM_LIBRA_COMMITMENTS);
        if has_zk {
            libra_commitments.push(transcript.receive_point_from_prover().unwrap());
            // "Libra:concatenation_commitment"
        }

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

        let (mut sum_check_round_failed, mut round_target_total_sum) = (false, ScalarField::zero());

        let mut libra_challenge = ScalarField::one();
        if has_zk {
            // If running zero-knowledge sumcheck the target total sum is corrected by the claimed sum of libra masking
            // multivariate over the hypercube

            let libra_total_sum = transcript.receive_fr_from_prover()?; // "Libra:Sum"
            libra_challenge = transcript.get_challenge::<H>(); // "Libra:Challenge"
            round_target_total_sum += libra_total_sum * libra_challenge;
        }

        // sumcheck round state
        let mut multivariate_challenge = Vec::with_capacity(CONST_PROOF_SIZE_LOG_N);

        let mut verified: bool = true;
        for (round_idx, &padding_value) in padding_indicator_array.iter().enumerate() {
            // Obtain the round univariate from the transcript
            let evaluations = transcript.receive_fr_array_from_verifier::<SIZE>()?;
            let round_univariate = SumcheckRoundOutput { evaluations };
            
            let round_challenge = transcript.get_challenge::<H>(); // format!("Sumcheck:u_{}", round_idx)
            multivariate_challenge.push(round_challenge);

            gate_separators.partially_evaluate(round_challenge, padding_indicator_array[round_idx]);

            let checked = Self::round_check_sum(
                &round_univariate,
                padding_value,
                &round_target_total_sum,
                &mut sum_check_round_failed,
            );
            Self::round_compute_next_target_sum(
                &round_univariate,
                round_challenge,
                padding_value,
                &mut round_target_total_sum,
            );

            verified = verified && checked;
        }

        // Extract claimed evaluations of Libra univariates and compute their sum multiplied by the Libra challenge
        // Final round
        let transcript_evaluations = transcript.receive_fr_vec_from_verifier(NUM_ALL_ENTITIES)?; // "Sumcheck:evaluations"
        for (eval, &transcript_eval) in self
            .memory
            .claimed_evaluations
            .iter_mut()
            .zip(transcript_evaluations.iter())
        {
            *eval = transcript_eval;
        }

        // Evaluate the Honk relation at the point (u_0, ..., u_{d-1}) using claimed evaluations of prover polynomials.
        let mut full_honk_purported_value =
            SumcheckVerifierRound::compute_full_relation_purported_value(
                &self.memory.claimed_evaluations,
                &self.memory.relation_parameters,
                &gate_separators.partial_evaluation_result,
            );

        // For ZK Flavors: the evaluation of the Row Disabling Polynomial at the sumcheck challenge
        let claimed_libra_evaluation = if has_zk {
            let libra_evaluation = transcript.receive_fr_from_prover()?; // "Libra:claimed_evaluation"
                                                                        
            let correcting_factor = RowDisablingPolynomial::evaluate_at_challenge_with_padding(
                &multivariate_challenge,
                &padding_indicator_array,
            );

            full_honk_purported_value =
                full_honk_purported_value * correcting_factor + libra_evaluation * libra_challenge;
            
            libra_evaluation
        } else {
            // Treated as "None"
            ScalarField::zero()
        };

        if has_zk {
            libra_commitments.push(transcript.receive_point_from_prover().unwrap()); // "Libra:grand_sum_commitment"
            libra_commitments.push(transcript.receive_point_from_prover().unwrap()); // "Libra:quotient_commitment"
        }

        Ok((
            SumcheckVerifierOutput {
                multivariate_challenge,
                verified: verified && full_honk_purported_value == round_target_total_sum,
                claimed_libra_evaluation,
            },
            libra_commitments,
        ))
    }

    // round state update functions
    pub(crate) fn round_compute_next_target_sum<const SIZE: usize>(
        univariate: &SumcheckRoundOutput<SIZE>,
        round_challenge: ScalarField,
        indicator: ScalarField,
        target_total_sum: &mut ScalarField,
    ) {
        *target_total_sum = (ScalarField::one() - indicator) * *target_total_sum
            + indicator * univariate.evaluate(round_challenge);
    }

    // sumcheck round check functions
    pub(crate) fn round_check_sum<const SIZE: usize>(
        univariate: &SumcheckRoundOutput<SIZE>,
        indicator: ScalarField,
        target_total_sum: &ScalarField,
        round_failed: &mut bool,
    ) -> bool {
        let total_sum = (ScalarField::one() - indicator) * *target_total_sum
            + indicator * univariate.evaluations[0]
            + univariate.evaluations[1];
        let sumcheck_round_failed = *target_total_sum != total_sum;

        *round_failed = *round_failed || sumcheck_round_failed;
        !sumcheck_round_failed
    }
}
