use crate::types::ScalarField;
use ark_ff::{One, Zero};

pub struct RowDisablingPolynomial {
    pub eval_at_0: ScalarField,
    pub eval_at_1: ScalarField,
}

impl Default for RowDisablingPolynomial {
    fn default() -> Self {
        Self {
            eval_at_0: ScalarField::one(),
            eval_at_1: ScalarField::one(),
        }
    }
}
impl RowDisablingPolynomial {
    pub fn update_evaluations(&mut self, round_challenge: ScalarField, round_idx: usize) {
        if round_idx == 1 {
            self.eval_at_0 = ScalarField::zero();
        }
        if round_idx >= 2 {
            self.eval_at_1 *= round_challenge;
        }
    }

    pub fn evaluate_at_challenge(
        multivariate_challenge: &[ScalarField],
        log_circuit_size: usize,
    ) -> ScalarField {
        let mut evaluation_at_multivariate_challenge = ScalarField::one();

        for val in multivariate_challenge.iter().take(log_circuit_size).skip(2) {
            evaluation_at_multivariate_challenge *= val;
        }

        ScalarField::one() - evaluation_at_multivariate_challenge
    }
    /**
     * @brief A variant of the above that uses `padding_indicator_array`.
     *
     * @param multivariate_challenge Sumcheck evaluation challenge
     * @param padding_indicator_array An array with first log_n entries equal to 1, and the remaining entries are 0.
     */
    pub fn evaluate_at_challenge_with_padding(
        multivariate_challenge: &[ScalarField],
        padding_indicator_array: &[ScalarField],
    ) -> ScalarField {
        let mut evaluation_at_multivariate_challenge = ScalarField::one();

        for (idx, &indicator) in padding_indicator_array.iter().enumerate().skip(2) {
            evaluation_at_multivariate_challenge *=
                ScalarField::one() - indicator + indicator * multivariate_challenge[idx];
        }

        ScalarField::one() - evaluation_at_multivariate_challenge
    }
}
