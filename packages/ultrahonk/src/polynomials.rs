use crate::types::ScalarField;
use ark_ff::One;

pub struct RowDisablingPolynomial {}

impl RowDisablingPolynomial {
    /// A variant of the above that uses `padding_indicator_array`.
    ///
    /// # Arguments
    ///
    /// * `multivariate_challenge` - Sumcheck evaluation challenge
    /// * `padding_indicator_array` - An array with first log_n entries equal to 1, and the remaining entries are 0.
    pub(crate) fn evaluate_at_challenge_with_padding(
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
