pub mod auxiliary_relation;
pub mod delta_range_constraint_relation;
pub mod elliptic_relation;
pub mod logderiv_lookup_relation;
pub mod permutation_relation;
pub mod poseidon2_external_relation;
pub mod poseidon2_internal_relation;
pub mod ultra_arithmetic_relation;

use super::super::types::{ClaimedEvaluations, RelationParameters};
use crate::types::ScalarField;
use ark_ff::Zero;
use auxiliary_relation::{AuxiliaryRelation, AuxiliaryRelationEvals};
use delta_range_constraint_relation::{
    DeltaRangeConstraintRelation, DeltaRangeConstraintRelationEvals,
};
use elliptic_relation::{EllipticRelation, EllipticRelationEvals};
use logderiv_lookup_relation::{LogDerivLookupRelation, LogDerivLookupRelationEvals};
use permutation_relation::{UltraPermutationRelation, UltraPermutationRelationEvals};
use poseidon2_external_relation::{Poseidon2ExternalRelation, Poseidon2ExternalRelationEvals};
use poseidon2_internal_relation::{Poseidon2InternalRelation, Poseidon2InternalRelationEvals};
use ultra_arithmetic_relation::{UltraArithmeticRelation, UltraArithmeticRelationEvals};

pub(crate) trait Relation {
    type VerifyAcc: Default;

    fn verify_accumulate(
        univariate_accumulator: &mut Self::VerifyAcc,
        input: &ClaimedEvaluations,
        relation_parameters: &RelationParameters,
        scaling_factor: &ScalarField,
    );
}

pub(crate) const NUM_SUBRELATIONS: usize = UltraArithmeticRelation::NUM_RELATIONS
    + UltraPermutationRelation::NUM_RELATIONS
    + DeltaRangeConstraintRelation::NUM_RELATIONS
    + EllipticRelation::NUM_RELATIONS
    + AuxiliaryRelation::NUM_RELATIONS
    + LogDerivLookupRelation::NUM_RELATIONS
    + Poseidon2ExternalRelation::NUM_RELATIONS
    + Poseidon2InternalRelation::NUM_RELATIONS;

#[derive(Default)]
pub(crate) struct AllRelationEvaluations {
    pub(crate) r_arith: UltraArithmeticRelationEvals,
    pub(crate) r_perm: UltraPermutationRelationEvals,
    pub(crate) r_lookup: LogDerivLookupRelationEvals,
    pub(crate) r_delta: DeltaRangeConstraintRelationEvals,
    pub(crate) r_elliptic: EllipticRelationEvals,
    pub(crate) r_aux: AuxiliaryRelationEvals,
    pub(crate) r_pos_ext: Poseidon2ExternalRelationEvals,
    pub(crate) r_pos_int: Poseidon2InternalRelationEvals,
}

impl AllRelationEvaluations {
    pub(crate) fn scale_and_batch_elements(
        &self,
        first_scalar: ScalarField,
        elements: &[ScalarField],
    ) -> ScalarField {
        assert!(elements.len() == NUM_SUBRELATIONS - 1);
        let mut output = ScalarField::zero();
        self.r_arith
            .scale_and_batch_elements(&[first_scalar, elements[0]], &mut output);
        self.r_perm
            .scale_and_batch_elements(&elements[1..3], &mut output);
        self.r_lookup
            .scale_and_batch_elements(&elements[3..5], &mut output);
        self.r_delta
            .scale_and_batch_elements(&elements[5..9], &mut output);
        self.r_elliptic
            .scale_and_batch_elements(&elements[9..11], &mut output);
        self.r_aux
            .scale_and_batch_elements(&elements[11..17], &mut output);

        self.r_pos_ext
            .scale_and_batch_elements(&elements[17..21], &mut output);
        self.r_pos_int
            .scale_and_batch_elements(&elements[21..], &mut output);

        output
    }
}
