use super::Relation;
use crate::alloc::borrow::ToOwned;
use crate::decider::{
    types::{ClaimedEvaluations, ProverUnivariates, RelationParameters},
    univariate::Univariate,
};
use crate::gadgets::poseidon2::POSEIDON2_BN254_T4_PARAMS;
use ark_ff::{BigInteger, PrimeField, Zero};

#[derive(Clone, Debug, Default)]
pub(crate) struct Poseidon2InternalRelationAcc<F: PrimeField> {
    pub(crate) r0: Univariate<F, 7>,
    pub(crate) r1: Univariate<F, 7>,
    pub(crate) r2: Univariate<F, 7>,
    pub(crate) r3: Univariate<F, 7>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Poseidon2InternalRelationEvals<F: PrimeField> {
    pub(crate) r0: F,
    pub(crate) r1: F,
    pub(crate) r2: F,
    pub(crate) r3: F,
}

impl<F: PrimeField> Poseidon2InternalRelationEvals<F> {
    pub(crate) fn scale_and_batch_elements(&self, running_challenge: &[F], result: &mut F) {
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

impl<F: PrimeField> Relation<F> for Poseidon2InternalRelation {
    type Acc = Poseidon2InternalRelationAcc<F>;
    type VerifyAcc = Poseidon2InternalRelationEvals<F>;

    fn verify_accumulate(
        univariate_accumulator: &mut Self::VerifyAcc,
        input: &ClaimedEvaluations<F>,
        _relation_parameters: &RelationParameters<F>,
        scaling_factor: &F,
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
        let internal_matrix_diag_0 = F::from_le_bytes_mod_order(
            &POSEIDON2_BN254_T4_PARAMS.mat_internal_diag_m_1[0]
                .into_bigint()
                .to_bytes_le(),
        );
        let internal_matrix_diag_1 = F::from_le_bytes_mod_order(
            &POSEIDON2_BN254_T4_PARAMS.mat_internal_diag_m_1[1]
                .into_bigint()
                .to_bytes_le(),
        );
        let internal_matrix_diag_2 = F::from_le_bytes_mod_order(
            &POSEIDON2_BN254_T4_PARAMS.mat_internal_diag_m_1[2]
                .into_bigint()
                .to_bytes_le(),
        );
        let internal_matrix_diag_3 = F::from_le_bytes_mod_order(
            &POSEIDON2_BN254_T4_PARAMS.mat_internal_diag_m_1[3]
                .into_bigint()
                .to_bytes_le(),
        );

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
