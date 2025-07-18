use super::Relation;
use crate::alloc::borrow::ToOwned;
use crate::decider::types::{ClaimedEvaluations, RelationParameters};
use crate::types::ScalarField;
use ark_ff::{AdditiveGroup, Field};

#[derive(Clone, Debug, Default)]
pub(crate) struct Poseidon2ExternalRelationEvals {
    pub(crate) r0: ScalarField,
    pub(crate) r1: ScalarField,
    pub(crate) r2: ScalarField,
    pub(crate) r3: ScalarField,
}

impl Poseidon2ExternalRelationEvals {
    pub(crate) fn scale_and_batch_elements(
        &self,
        running_challenge: &[ScalarField],
        result: &mut ScalarField,
    ) {
        assert!(running_challenge.len() == Poseidon2ExternalRelation::NUM_RELATIONS);

        *result += self.r0 * running_challenge[0];
        *result += self.r1 * running_challenge[1];
        *result += self.r2 * running_challenge[2];
        *result += self.r3 * running_challenge[3];
    }
}

pub(crate) struct Poseidon2ExternalRelation {}

impl Poseidon2ExternalRelation {
    pub(crate) const NUM_RELATIONS: usize = 4;
}

impl Relation for Poseidon2ExternalRelation {
    type VerifyAcc = Poseidon2ExternalRelationEvals;

    /// Expression for the poseidon2 external round relation, based on E_i in Section 6 of
    /// <https://eprint.iacr.org/2023/323.pdf>.
    ///
    /// This relation is defined as C(in(X)...) :=
    /// q_poseidon2_external * ( (v1 - w_1_shift) + \alpha * (v2 - w_2_shift) +
    /// \alpha^2 * (v3 - w_3_shift) + \alpha^3 * (v4 - w_4_shift) ) = 0 where:
    ///      u1 := (w_1 + q_1)^5
    ///      u2 := (w_2 + q_2)^5
    ///      u3 := (w_3 + q_3)^5
    ///      u4 := (w_4 + q_4)^5
    ///      t0 := u1 + u2                                           (1, 1, 0, 0)
    ///      t1 := u3 + u4                                           (0, 0, 1, 1)
    ///      t2 := 2 * u2 + t1 = 2 * u2 + u3 + u4                    (0, 2, 1, 1)
    ///      t3 := 2 * u4 + t0 = u1 + u2 + 2 * u4                    (1, 1, 0, 2)
    ///      v4 := 4 * t1 + t3 = u1 + u2 + 4 * u3 + 6 * u4           (1, 1, 4, 6)
    ///      v2 := 4 * t0 + t2 = 4 * u1 + 6 * u2 + u3 + u4           (4, 6, 1, 1)
    ///      v1 := t3 + v2 = 5 * u1 + 7 * u2 + 1 * u3 + 3 * u4       (5, 7, 1, 3)
    ///      v3 := t2 + v4                                           (1, 3, 5, 7)
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
        let q_r = input.precomputed.q_r();
        let q_o = input.precomputed.q_o();
        let q_4 = input.precomputed.q_4();
        let q_poseidon2_external = input.precomputed.q_poseidon2_external();

        // add round constants which are loaded in selectors
        let s1 = w_l.to_owned() + q_l;
        let s2 = w_r.to_owned() + q_r;
        let s3 = w_o.to_owned() + q_o;
        let s4 = w_4.to_owned() + q_4;

        // apply s-box round
        let mut u1 = s1.to_owned().square();
        u1 = u1.square();
        u1 *= s1;
        let mut u2 = s2.to_owned().square();
        u2 = u2.square();
        u2 *= s2;
        let mut u3 = s3.to_owned().square();
        u3 = u3.square();
        u3 *= s3;
        let mut u4 = s4.to_owned().square();
        u4 = u4.square();
        u4 *= s4;

        // matrix mul v = M_E * u with 14 additions
        let t0 = u1 + u2; // u_1 + u_2
        let t1 = u3 + u4; // u_3 + u_4
        let mut t2 = u2.double(); // 2u_2
        t2 += &t1; // 2u_2 + u_3 + u_4
        let mut t3 = u4.double(); // 2u_4
        t3 += &t0; // u_1 + u_2 + 2u_4
        let mut v4 = t1.double();
        v4.double_in_place();
        v4 += &t3; // u_1 + u_2 + 4u_3 + 6u_4
        let mut v2 = t0.double();
        v2.double_in_place();
        v2 += &t2; // 4u_1 + 6u_2 + u_3 + u_4
        let v1 = t3 + v2; // 5u_1 + 7u_2 + u_3 + 3u_4
        let v3 = t2 + v4; // u_1 + 3u_2 + 5u_3 + 7u_4

        let q_pos_by_scaling = q_poseidon2_external.to_owned() * scaling_factor;
        let tmp = (v1 - w_l_shift) * q_pos_by_scaling;
        univariate_accumulator.r0 += tmp;

        ///////////////////////////////////////////////////////////////////////

        let tmp = (v2 - w_r_shift) * q_pos_by_scaling;
        univariate_accumulator.r1 += tmp;

        ///////////////////////////////////////////////////////////////////////

        let tmp = (v3 - w_o_shift) * q_pos_by_scaling;
        univariate_accumulator.r2 += tmp;

        ///////////////////////////////////////////////////////////////////////

        let tmp = (v4 - w_4_shift) * q_pos_by_scaling;
        univariate_accumulator.r3 += tmp;
    }
}
