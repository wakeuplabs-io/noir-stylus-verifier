use super::Relation;
use crate::alloc::borrow::ToOwned;
use crate::decider::types::{ClaimedEvaluations, RelationParameters};
use crate::types::ScalarField;
use ark_ff::Field;

#[derive(Clone, Debug, Default)]
pub(crate) struct UltraArithmeticRelationEvals {
    pub(crate) r0: ScalarField,
    pub(crate) r1: ScalarField,
}

impl UltraArithmeticRelationEvals {
    pub(crate) fn scale_and_batch_elements(
        &self,
        running_challenge: &[ScalarField],
        result: &mut ScalarField,
    ) {
        assert!(running_challenge.len() == UltraArithmeticRelation::NUM_RELATIONS);

        *result += self.r0 * running_challenge[0];
        *result += self.r1 * running_challenge[1];
    }
}

pub(crate) struct UltraArithmeticRelation {}

impl UltraArithmeticRelation {
    pub(crate) const NUM_RELATIONS: usize = 2;
}

impl Relation for UltraArithmeticRelation {
    type VerifyAcc = UltraArithmeticRelationEvals;

    fn verify_accumulate(
        univariate_accumulator: &mut Self::VerifyAcc,
        input: &ClaimedEvaluations,
        _relation_parameters: &RelationParameters,
        scaling_factor: &ScalarField,
    ) {
        let w_l = input.witness.w_l();
        let w_r = input.witness.w_r();
        let w_o = input.witness.w_o();
        let w_4 = input.witness.w_4();
        let w_4_shift = input.shifted_witness.w_4();
        let q_m = input.precomputed.q_m();
        let q_l = input.precomputed.q_l();
        let q_r = input.precomputed.q_r();
        let q_o = input.precomputed.q_o();
        let q_4 = input.precomputed.q_4();
        let q_c = input.precomputed.q_c();
        let q_arith = input.precomputed.q_arith();
        let w_l_shift = input.shifted_witness.w_l();

        let neg_half = -ScalarField::from(2u64).inverse().unwrap();

        let mut tmp: ScalarField = (q_arith.to_owned() - ScalarField::from(3_u64))
            * (q_m.to_owned() * w_r * w_l)
            * neg_half;
        tmp += (q_l.to_owned() * w_l)
            + (q_r.to_owned() * w_r)
            + (q_o.to_owned() * w_o)
            + (q_4.to_owned() * w_4)
            + q_c;
        tmp += (q_arith.to_owned() - ScalarField::from(1_u64)) * w_4_shift;
        tmp *= q_arith;
        tmp *= scaling_factor;

        univariate_accumulator.r0 += tmp;

        ///////////////////////////////////////////////////////////////////////

        let mut tmp = w_l.to_owned() + w_4 - w_l_shift + q_m;
        tmp *= q_arith.to_owned() - ScalarField::from(2_u64);
        tmp *= q_arith.to_owned() - ScalarField::from(1_u64);
        tmp *= q_arith;
        tmp *= scaling_factor;

        univariate_accumulator.r1 += tmp;
    }
}
