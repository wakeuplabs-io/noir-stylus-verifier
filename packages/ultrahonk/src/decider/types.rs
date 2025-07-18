use crate::types::{G1Affine, ScalarField};
use crate::{types::AllEntities, NUM_ALPHAS};
use alloc::vec::Vec;
use ark_ff::One;

pub type ClaimedEvaluations = AllEntities<ScalarField>;
pub type VerifierCommitments = AllEntities<G1Affine>;

#[derive(Clone)]
#[cfg_attr(test, derive(Debug, Default, PartialEq, Eq))]
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

pub(crate) struct GateSeparatorPolynomial {
    betas: Vec<ScalarField>,
    pub(crate) partial_evaluation_result: ScalarField,
    current_element_idx: usize,
    pub(crate) periodicity: usize,
}

impl GateSeparatorPolynomial {
    pub(crate) fn new_without_products(betas: Vec<ScalarField>) -> Self {
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

    /// Partially evaluate the \f$pow_{\beta} \f$-polynomial at the new challenge and update \f$ c_i \f$
    /// Update the constant \f$c_{i} \to c_{i+1} \f$ multiplying it by \f$pow_{\beta}\f$'s factor \f$\left(
    /// (1-X_i) + X_i\cdot \beta_i\right)\vert_{X_i = u_i}\f$ computed by \ref univariate_eval.
    ///
    /// # Arguments
    ///
    /// * `round_challenge` - \f$ i \f$-th verifier challenge \f$ u_{i}\f$
    /// * `indicator` - An entry of `padding_indicator_array`, which is equal to 1 when round_idx < log_circuit_size and is 0 otherwise.
    ///
    pub(crate) fn partially_evaluate(
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
