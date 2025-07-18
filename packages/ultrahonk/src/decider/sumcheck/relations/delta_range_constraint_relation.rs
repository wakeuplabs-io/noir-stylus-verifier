use super::Relation;
use crate::alloc::borrow::ToOwned;
use crate::decider::types::{ClaimedEvaluations, RelationParameters};
use crate::types::ScalarField;
use ark_ff::{Field, One};

#[derive(Clone, Debug, Default)]
pub(crate) struct DeltaRangeConstraintRelationEvals {
    pub(crate) r0: ScalarField,
    pub(crate) r1: ScalarField,
    pub(crate) r2: ScalarField,
    pub(crate) r3: ScalarField,
}

impl DeltaRangeConstraintRelationEvals {
    pub(crate) fn scale_and_batch_elements(
        &self,
        running_challenge: &[ScalarField],
        result: &mut ScalarField,
    ) {
        assert!(running_challenge.len() == DeltaRangeConstraintRelation::NUM_RELATIONS);

        *result += self.r0 * running_challenge[0];
        *result += self.r1 * running_challenge[1];
        *result += self.r2 * running_challenge[2];
        *result += self.r3 * running_challenge[3];
    }
}

pub(crate) struct DeltaRangeConstraintRelation {}

impl DeltaRangeConstraintRelation {
    pub(crate) const NUM_RELATIONS: usize = 4;
}

impl Relation for DeltaRangeConstraintRelation {
    type VerifyAcc = DeltaRangeConstraintRelationEvals;

    /// Expression for the generalized permutation sort gate.
    ///
    /// The relation is defined as C(in(X)...) =
    ///    q_delta_range * \sum{ i = [0, 3]} \alpha^i D_i(D_i - 1)(D_i - 2)(D_i - 3)
    ///       where
    ///       D_0 = w_2 - w_1
    ///       D_1 = w_3 - w_2
    ///       D_2 = w_4 - w_3
    ///       D_3 = w_1_shift - w_4
    ///
    /// # Arguments
    ///
    /// * `univariate_accumulator` transformed to `univariate_accumulator + C(in(X)...)*scaling_factor`
    /// * `input` an std::array containing the fully extended Univariate edges.
    /// * `relation_parameters` contains beta, gamma, and public_input_delta, ....
    /// * `scaling_factor` optional term to scale the evaluation before adding to evals.
    ///
    fn accumulate(
        univariate_accumulator: &mut Self::VerifyAcc,
        input: &ClaimedEvaluations,
        _relation_parameters: &RelationParameters,
        scaling_factor: &ScalarField,
    ) {
        let w_1 = input.witness.w_l();
        let w_2 = input.witness.w_r();
        let w_3 = input.witness.w_o();
        let w_4 = input.witness.w_4();
        let w_1_shift = input.shifted_witness.w_l();
        let q_delta_range = input.precomputed.q_delta_range();
        let minus_one = -ScalarField::one();
        let minus_two = -ScalarField::from(2u64);

        // Compute wire differences
        let delta_1 = w_2.to_owned() - w_1;
        let delta_2 = w_3.to_owned() - w_2;
        let delta_3 = w_4.to_owned() - w_3;
        let delta_4 = w_1_shift.to_owned() - w_4;

        // Contribution (1)
        let mut tmp_1 = (delta_1.to_owned() + minus_one).square() + minus_one;
        tmp_1 *= (delta_1.to_owned() + minus_two).square() + minus_one;
        tmp_1 *= q_delta_range;
        tmp_1 *= scaling_factor;

        univariate_accumulator.r0 += tmp_1;

        ///////////////////////////////////////////////////////////////////////
        // Contribution (2)
        let mut tmp_2 = (delta_2.to_owned() + minus_one).square() + minus_one;
        tmp_2 *= (delta_2.to_owned() + minus_two).square() + minus_one;
        tmp_2 *= q_delta_range;
        tmp_2 *= scaling_factor;

        univariate_accumulator.r1 += tmp_2;

        ///////////////////////////////////////////////////////////////////////
        // Contribution (3)
        let mut tmp_3 = (delta_3.to_owned() + minus_one).square() + minus_one;
        tmp_3 *= (delta_3.to_owned() + minus_two).square() + minus_one;
        tmp_3 *= q_delta_range;
        tmp_3 *= scaling_factor;

        univariate_accumulator.r2 += tmp_3;

        ///////////////////////////////////////////////////////////////////////
        // Contribution (4)
        let mut tmp_4 = (delta_4.to_owned() + minus_one).square() + minus_one;
        tmp_4 *= (delta_4.to_owned() + minus_two).square() + minus_one;
        tmp_4 *= q_delta_range;
        tmp_4 *= scaling_factor;

        univariate_accumulator.r3 += tmp_4;
    }
}
