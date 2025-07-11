use super::Relation;
use crate::alloc::borrow::ToOwned;
use crate::decider::types::{ClaimedEvaluations, RelationParameters};
use crate::types::ScalarField;
use crate::Utils;
use ark_ff::Field;

#[derive(Clone, Debug, Default)]
pub(crate) struct Poseidon2InternalRelationEvals {
    pub(crate) r0: ScalarField,
    pub(crate) r1: ScalarField,
    pub(crate) r2: ScalarField,
    pub(crate) r3: ScalarField,
}

impl Poseidon2InternalRelationEvals {
    pub(crate) fn scale_and_batch_elements(
        &self,
        running_challenge: &[ScalarField],
        result: &mut ScalarField,
    ) {
        assert!(running_challenge.len() == Poseidon2InternalRelation::NUM_RELATIONS);

        *result += self.r0 * running_challenge[0];
        *result += self.r1 * running_challenge[1];
        *result += self.r2 * running_challenge[2];
        *result += self.r3 * running_challenge[3];
    }
}

pub(crate) struct Poseidon2InternalRelation {}

impl Poseidon2InternalRelation {
    pub(crate) const NUM_RELATIONS: usize = 4;
}

impl Relation for Poseidon2InternalRelation {
    type VerifyAcc = Poseidon2InternalRelationEvals;

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
        let w_l_shift = input.shifted_witness.w_l();
        let w_r_shift = input.shifted_witness.w_r();
        let w_o_shift = input.shifted_witness.w_o();
        let w_4_shift = input.shifted_witness.w_4();
        let q_l = input.precomputed.q_l();
        let q_poseidon2_internal = input.precomputed.q_poseidon2_internal();

        // add round constants
        let s1 = w_l.to_owned() + q_l;

        // apply s-box round
        let mut u1 = s1.to_owned().square();
        u1 = u1.square();
        u1 *= s1;
        let u2 = w_r.to_owned();
        let u3 = w_o.to_owned();
        let u4 = w_4.to_owned();

        // matrix mul with v = M_I * u 4 muls and 7 additions
        let sum = u1.to_owned() + u2 + u3 + u4;

        let q_pos_by_scaling = q_poseidon2_internal.to_owned() * scaling_factor;

        // TACEO TODO this poseidon instance is very hardcoded to the bn254 curve
        let internal_matrix_diag_0 = Utils::field_from_hex_string(
            "0x10dc6e9c006ea38b04b1e03b4bd9490c0d03f98929ca1d7fb56821fd19d3b6e7",
        )
        .unwrap();
        let internal_matrix_diag_1 = Utils::field_from_hex_string(
            "0x0c28145b6a44df3e0149b3d0a30b3bb599df9756d4dd9b84a86b38cfb45a740b",
        )
        .unwrap();
        let internal_matrix_diag_2 = Utils::field_from_hex_string(
            "0x00544b8338791518b2c7645a50392798b21f75bb60e3596170067d00141cac15",
        )
        .unwrap();
        let internal_matrix_diag_3 = Utils::field_from_hex_string(
            "0x222c01175718386f2e2e82eb122789e352e105a3b8fa852613bc534433ee428b",
        )
        .unwrap();

        let mut v1 = u1 * internal_matrix_diag_0;
        v1 += &sum;
        let tmp = (v1 - w_l_shift) * q_pos_by_scaling;

        univariate_accumulator.r0 += tmp;

        ///////////////////////////////////////////////////////////////////////

        let mut v2 = u2 * internal_matrix_diag_1;
        v2 += &sum;
        let tmp = (v2 - w_r_shift) * q_pos_by_scaling;

        univariate_accumulator.r1 += tmp;

        ///////////////////////////////////////////////////////////////////////

        let mut v3 = u3 * internal_matrix_diag_2;
        v3 += &sum;
        let tmp = (v3 - w_o_shift) * q_pos_by_scaling;

        univariate_accumulator.r2 += tmp;

        ///////////////////////////////////////////////////////////////////////

        let mut v4 = u4 * internal_matrix_diag_3;
        v4 += sum;
        let tmp = (v4 - w_4_shift) * q_pos_by_scaling;

        univariate_accumulator.r3 += tmp;
    }
}
