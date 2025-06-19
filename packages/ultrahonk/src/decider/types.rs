use crate::keys::verification_key::VerifyingKey;
use crate::types::{G1Affine, ScalarField};
use crate::{types::AllEntities, NUM_ALPHAS};
use alloc::{vec::Vec};
use ark_ff::PrimeField;

pub(crate) struct VerifierMemory {
    pub(crate) verifier_commitments: VerifierCommitments<G1Affine>,
    pub(crate) relation_parameters: RelationParameters<ScalarField>,
    pub(crate) claimed_evaluations: ClaimedEvaluations<ScalarField>,
}

pub(crate) const MAX_PARTIAL_RELATION_LENGTH: usize = 7;
pub(crate) const BATCHED_RELATION_PARTIAL_LENGTH: usize = MAX_PARTIAL_RELATION_LENGTH + 1;

pub(crate) type ClaimedEvaluations<F> = AllEntities<F>;
pub(crate) type VerifierCommitments<P> = AllEntities<P>;

pub(crate) struct RelationParameters<F: PrimeField> {
    pub(crate) eta_1: F,
    pub(crate) eta_2: F,
    pub(crate) eta_3: F,
    pub(crate) beta: F,
    pub(crate) gamma: F,
    pub(crate) public_input_delta: F,
    pub(crate) alphas: [F; NUM_ALPHAS],
    pub(crate) gate_challenges: Vec<F>,
}

pub struct GateSeparatorPolynomial<F: PrimeField> {
    betas: Vec<F>,
    pub partial_evaluation_result: F,
    current_element_idx: usize,
    pub periodicity: usize,
}

impl<F: PrimeField> GateSeparatorPolynomial<F> {
    pub fn new_without_products(betas: Vec<F>) -> Self {
        let current_element_idx = 0;
        let periodicity = 2;
        let partial_evaluation_result = F::ONE;

        Self {
            betas,
            partial_evaluation_result,
            current_element_idx,
            periodicity,
        }
    }
    pub fn partially_evaluate_with_padding(&mut self, round_challenge: F, indicator: F) {
        let current_univariate_eval =
            F::ONE + (round_challenge * (self.betas[self.current_element_idx] - F::ONE));
        // If dummy round, make no update to the partial_evaluation_result
        self.partial_evaluation_result = (F::ONE - indicator) * self.partial_evaluation_result
            + indicator * self.partial_evaluation_result * current_univariate_eval;
        self.current_element_idx += 1;
        self.periodicity *= 2;
    }
}

impl VerifierMemory {
    #[expect(clippy::field_reassign_with_default)]
    pub(crate) fn from_memory_and_key(
        verifier_memory: crate::oink::types::VerifierMemory,
        vk: &VerifyingKey,
    ) -> Self {
        let relation_parameters = RelationParameters {
            eta_1: verifier_memory.challenges.eta_1,
            eta_2: verifier_memory.challenges.eta_2,
            eta_3: verifier_memory.challenges.eta_3,
            beta: verifier_memory.challenges.beta,
            gamma: verifier_memory.challenges.gamma,
            public_input_delta: verifier_memory.public_input_delta,
            alphas: verifier_memory.challenges.alphas,
            gate_challenges: Default::default(),
        };

        let mut memory = AllEntities::default();
        memory.witness = verifier_memory.witness_commitments;
        memory.precomputed = vk.commitments.clone();

        // These copies are not required
        // for (des, src) in izip!(
        //     memory.shifted_witness.iter_mut(),
        //     memory.witness.to_be_shifted().iter().cloned(),
        // ) {
        //     *des = src;
        // }
        // for (des, src) in izip!(
        //     memory.shifted_tables.iter_mut(),
        //     memory.precomputed.get_table_polynomials().iter().cloned()
        // ) {
        //     *des = src;
        // }

        Self {
            relation_parameters,
            verifier_commitments: memory,
            claimed_evaluations: Default::default(),
        }
    }
}
