use super::Relation;
use crate::alloc::borrow::ToOwned;
use crate::decider::types::{ClaimedEvaluations, RelationParameters};
use crate::serialize::BytesDeserializable;
use crate::types::ScalarField;
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

    /// Expression for the poseidon2 internal round relation, based on I_i in Section 6 of
    /// <https://eprint.iacr.org/2023/323.pdf>.
    ///
    /// This relation is defined as C(in(X)...) :=
    /// q_poseidon2_internal * ( (v1 - w_1_shift) + \alpha * (v2 - w_2_shift) +
    /// \alpha^2 * (v3 - w_3_shift) + \alpha^3 * (v4 - w_4_shift) ) = 0 where:
    ///      u1 := (w_1 + q_1)^5
    ///      sum := u1 + w_2 + w_3 + w_4
    ///      v1 := u1 * D1 + sum
    ///      v2 := w_2 * D2 + sum
    ///      v3 := w_3 * D3 + sum
    ///      v4 := w_4 * D4 + sum
    ///      Di is the ith internal diagonal value - 1 of the internal matrix M_I
    ///
    /// # Arguments
    ///
    /// * `univariate_accumulator` transformed to `univariate_accumulator + C(in(X)...)*scaling_factor`
    /// * `input` an std::array containing the fully extended Univariate edges.
    /// * `relation_parameters` contains beta, gamma, and public_input_delta, ....
    /// * `scaling_factor` optional term to scale the evaluation before adding to evals.
    fn accumulate(
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

        // This poseidon instance is hardcoded to the bn254 curve
        let internal_matrix_diag_0 = ScalarField::deserialize_from_bytes(
            // hex::decode("10dc6e9c006ea38b04b1e03b4bd9490c0d03f98929ca1d7fb56821fd19d3b6e7")
            &[
                16, 220, 110, 156, 0, 110, 163, 139, 4, 177, 224, 59, 75, 217, 73, 12, 13, 3, 249,
                137, 41, 202, 29, 127, 181, 104, 33, 253, 25, 211, 182, 231,
            ],
        )
        .unwrap()
        .0;
        let internal_matrix_diag_1 = ScalarField::deserialize_from_bytes(
            // hex::decode("0c28145b6a44df3e0149b3d0a30b3bb599df9756d4dd9b84a86b38cfb45a740b")
            &[
                12, 40, 20, 91, 106, 68, 223, 62, 1, 73, 179, 208, 163, 11, 59, 181, 153, 223, 151,
                86, 212, 221, 155, 132, 168, 107, 56, 207, 180, 90, 116, 11,
            ],
        )
        .unwrap()
        .0;
        let internal_matrix_diag_2 = ScalarField::deserialize_from_bytes(
            // hex::decode("00544b8338791518b2c7645a50392798b21f75bb60e3596170067d00141cac15")
            &[
                0, 84, 75, 131, 56, 121, 21, 24, 178, 199, 100, 90, 80, 57, 39, 152, 178, 31, 117,
                187, 96, 227, 89, 97, 112, 6, 125, 0, 20, 28, 172, 21,
            ],
        )
        .unwrap()
        .0;
        let internal_matrix_diag_3 = ScalarField::deserialize_from_bytes(
            // hex::decode("222c01175718386f2e2e82eb122789e352e105a3b8fa852613bc534433ee428b")
            &[
                34, 44, 1, 23, 87, 24, 56, 111, 46, 46, 130, 235, 18, 39, 137, 227, 82, 225, 5,
                163, 184, 250, 133, 38, 19, 188, 83, 68, 51, 238, 66, 139,
            ],
        )
        .unwrap()
        .0;

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
