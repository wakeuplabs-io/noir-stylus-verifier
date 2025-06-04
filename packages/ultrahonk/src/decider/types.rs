use super::univariate::Univariate;
use crate::{types::AllEntities, NUM_ALPHAS};
use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use crate::keys::verification_key::VerifyingKey;
use std::{vec};

pub(crate) struct VerifierMemory<P: Pairing> {
    pub(crate) verifier_commitments: VerifierCommitments<P::G1Affine>,
    pub(crate) relation_parameters: RelationParameters<P::ScalarField>,
    pub(crate) claimed_evaluations: ClaimedEvaluations<P::ScalarField>,
}

pub(crate) const MAX_PARTIAL_RELATION_LENGTH: usize = 7;
pub(crate) const BATCHED_RELATION_PARTIAL_LENGTH: usize = MAX_PARTIAL_RELATION_LENGTH + 1;
pub(crate) const BATCHED_RELATION_PARTIAL_LENGTH_ZK: usize = BATCHED_RELATION_PARTIAL_LENGTH + 1;

pub(crate) type ProverUnivariates<F> = AllEntities<Univariate<F, MAX_PARTIAL_RELATION_LENGTH>>;
pub(crate) type PartiallyEvaluatePolys<F> = AllEntities<Vec<F>>;
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
    pub beta_products: Vec<F>,
    pub partial_evaluation_result: F,
    current_element_idx: usize,
    pub periodicity: usize,
}

impl<F: PrimeField> GateSeparatorPolynomial<F> {
    pub fn new(betas: Vec<F>, log_num_mononmials: usize) -> Self {
        let pow_size = 1 << log_num_mononmials;
        let current_element_idx = 0;
        let periodicity = 2;
        let partial_evaluation_result = F::ONE;

        // Barretenberg uses multithreading here and a simpler algorithm with worse complexity
        let mut beta_products = vec![F::ONE; pow_size];
        for (i, beta) in betas.iter().take(log_num_mononmials).enumerate() {
            let index = 1 << i;
            beta_products[index] = *beta;
            for j in 1..index {
                beta_products[index + j] = beta_products[j] * beta;
            }
        }

        Self {
            betas,
            beta_products,
            partial_evaluation_result,
            current_element_idx,
            periodicity,
        }
    }

    pub fn new_without_products(betas: Vec<F>) -> Self {
        let current_element_idx = 0;
        let periodicity = 2;
        let partial_evaluation_result = F::ONE;

        Self {
            betas,
            beta_products: Vec::new(),
            partial_evaluation_result,
            current_element_idx,
            periodicity,
        }
    }

    pub fn current_element(&self) -> F {
        self.betas[self.current_element_idx]
    }

    pub fn partially_evaluate(&mut self, round_challenge: F) {
        let current_univariate_eval =
            F::ONE + (round_challenge * (self.betas[self.current_element_idx] - F::ONE));
        self.partial_evaluation_result *= current_univariate_eval;
        self.current_element_idx += 1;
        self.periodicity *= 2;
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

impl<P: Pairing> VerifierMemory<P> {
    #[expect(clippy::field_reassign_with_default)]
    pub(crate) fn from_memory_and_key(
        verifier_memory: crate::oink::types::VerifierMemory<P>,
        vk: &VerifyingKey<P>,
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
