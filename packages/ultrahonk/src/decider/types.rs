use crate::backends::{G1ArithmeticBackend, HashBackend};
use crate::keys::verification_key::VerifyingKey;
use crate::oink::verifier::OinkVerifier;
use crate::transcript::Transcript;
use crate::types::{G1Affine, ScalarField};
use crate::CONST_PROOF_SIZE_LOG_N;
use crate::{types::AllEntities, NUM_ALPHAS};
use alloc::vec::Vec;
use ark_ff::One;

pub struct VerifierMemory {
    pub verifier_commitments: VerifierCommitments,
    pub relation_parameters: RelationParameters,
    pub claimed_evaluations: ClaimedEvaluations,
}

pub const MAX_PARTIAL_RELATION_LENGTH: usize = 7;
pub const BATCHED_RELATION_PARTIAL_LENGTH: usize = MAX_PARTIAL_RELATION_LENGTH + 1;

pub type ClaimedEvaluations = AllEntities<ScalarField>;
pub type VerifierCommitments = AllEntities<G1Affine>;

pub struct RelationParameters {
    pub eta_1: ScalarField,
    pub eta_2: ScalarField,
    pub eta_3: ScalarField,
    pub beta: ScalarField,
    pub gamma: ScalarField,
    pub public_input_delta: ScalarField,
    pub alphas: [ScalarField; NUM_ALPHAS],
    pub gate_challenges: Vec<ScalarField>,
}

pub struct GateSeparatorPolynomial {
    betas: Vec<ScalarField>,
    pub partial_evaluation_result: ScalarField,
    current_element_idx: usize,
    pub periodicity: usize,
}

impl GateSeparatorPolynomial {
    pub fn new_without_products(betas: Vec<ScalarField>) -> Self {
        let current_element_idx = 0;
        let periodicity = 2;
        let partial_evaluation_result = ScalarField::one();

        Self {
            betas,
            partial_evaluation_result,
            current_element_idx,
            periodicity,
        }
    }
    pub fn partially_evaluate_with_padding(
        &mut self,
        round_challenge: ScalarField,
        indicator: ScalarField,
    ) {
        let current_univariate_eval = ScalarField::one()
            + (round_challenge * (self.betas[self.current_element_idx] - ScalarField::one()));
        // If dummy round, make no update to the partial_evaluation_result
        self.partial_evaluation_result = (ScalarField::one() - indicator)
            * self.partial_evaluation_result
            + indicator * self.partial_evaluation_result * current_univariate_eval;
        self.current_element_idx += 1;
        self.periodicity *= 2;
    }
}

impl VerifierMemory {
    pub fn from_key_and_transcript<P: G1ArithmeticBackend, H: HashBackend>(vk: &VerifyingKey, transcript: &mut Transcript) -> Self {
        let oink_verifier = OinkVerifier::<P>::default();
        let oink_result = oink_verifier.build_memory::<H>(vk, transcript).unwrap();

        // generate gate challenges
        let mut gate_challenges: Vec<ScalarField> = Vec::with_capacity(CONST_PROOF_SIZE_LOG_N);

        for idx in 0..CONST_PROOF_SIZE_LOG_N {
            let chall = transcript.get_challenge::<H>(format!("Sumcheck:gate_challenge_{}", idx));
            gate_challenges.push(chall);
        }

        let relation_parameters = RelationParameters {
            eta_1: oink_result.challenges.eta_1,
            eta_2: oink_result.challenges.eta_2,
            eta_3: oink_result.challenges.eta_3,
            beta: oink_result.challenges.beta,
            gamma: oink_result.challenges.gamma,
            public_input_delta: oink_result.public_input_delta,
            alphas: oink_result.challenges.alphas,
            gate_challenges,
        };

        let mut memory = AllEntities::default();
        memory.witness = oink_result.witness_commitments;
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
