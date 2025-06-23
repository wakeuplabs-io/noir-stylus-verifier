use super::super::univariate::Univariate;
use crate::{
    decider::{
        sumcheck::relations::{
            auxiliary_relation::AuxiliaryRelation,
            delta_range_constraint_relation::DeltaRangeConstraintRelation,
            elliptic_relation::{EllipticRelation, EllipticRelationEvals},
            logderiv_lookup_relation::LogDerivLookupRelation,
            permutation_relation::UltraPermutationRelation,
            poseidon2_external_relation::Poseidon2ExternalRelation,
            poseidon2_internal_relation::Poseidon2InternalRelation,
            ultra_arithmetic_relation::UltraArithmeticRelation,
            AllRelationEvaluations, Relation,
        },
        types::{ClaimedEvaluations, GateSeparatorPolynomial, RelationParameters},
    },
    types::ScalarField,
};
use ark_ff::{One, Zero};

pub(crate) type SumcheckRoundOutput<const U: usize> = Univariate<U>;

pub(crate) struct SumcheckVerifierRound {
    pub(crate) target_total_sum: ScalarField,
    pub(crate) round_failed: bool,
}

impl Default for SumcheckVerifierRound {
    fn default() -> Self {
        Self::new()
    }
}

impl SumcheckVerifierRound {
    pub(crate) fn new() -> Self {
        Self {
            target_total_sum: ScalarField::zero(),
            round_failed: false,
        }
    }

    pub(crate) fn compute_next_target_sum<const SIZE: usize>(
        &mut self,
        univariate: &SumcheckRoundOutput<SIZE>,
        round_challenge: ScalarField,
        indicator: ScalarField,
    ) {
        self.target_total_sum = (ScalarField::one() - indicator) * self.target_total_sum
            + indicator * univariate.evaluate(round_challenge);
    }

    pub(crate) fn check_sum<const SIZE: usize>(
        &mut self,
        univariate: &SumcheckRoundOutput<SIZE>,
        indicator: ScalarField,
    ) -> bool {
        let total_sum = (ScalarField::one() - indicator) * self.target_total_sum
            + indicator * univariate.evaluations[0]
            + univariate.evaluations[1];
        let sumcheck_round_failed = self.target_total_sum != total_sum;

        self.round_failed = self.round_failed || sumcheck_round_failed;
        !sumcheck_round_failed
    }

    fn accumulate_one_relation_evaluations<R: Relation>(
        univariate_accumulator: &mut R::VerifyAcc,
        extended_edges: &ClaimedEvaluations,
        relation_parameters: &RelationParameters,
        scaling_factor: &ScalarField,
    ) {
        R::verify_accumulate(
            univariate_accumulator,
            extended_edges,
            relation_parameters,
            scaling_factor,
        );
    }

    fn accumulate_elliptic_curve_relation_evaluations(
        univariate_accumulator: &mut EllipticRelationEvals,
        extended_edges: &ClaimedEvaluations,
        relation_parameters: &RelationParameters,
        scaling_factor: &ScalarField,
    ) {
        EllipticRelation::verify_accumulate(
            univariate_accumulator,
            extended_edges,
            relation_parameters,
            scaling_factor,
        );
    }

    fn accumulate_relation_evaluations(
        univariate_accumulators: &mut AllRelationEvaluations,
        extended_edges: &ClaimedEvaluations,
        relation_parameters: &RelationParameters,
        scaling_factor: &ScalarField,
    ) {
        Self::accumulate_one_relation_evaluations::<UltraArithmeticRelation>(
            &mut univariate_accumulators.r_arith,
            extended_edges,
            relation_parameters,
            scaling_factor,
        );
        Self::accumulate_one_relation_evaluations::<UltraPermutationRelation>(
            &mut univariate_accumulators.r_perm,
            extended_edges,
            relation_parameters,
            scaling_factor,
        );
        Self::accumulate_one_relation_evaluations::<DeltaRangeConstraintRelation>(
            &mut univariate_accumulators.r_delta,
            extended_edges,
            relation_parameters,
            scaling_factor,
        );
        Self::accumulate_elliptic_curve_relation_evaluations(
            &mut univariate_accumulators.r_elliptic,
            extended_edges,
            relation_parameters,
            scaling_factor,
        );
        Self::accumulate_one_relation_evaluations::<AuxiliaryRelation>(
            &mut univariate_accumulators.r_aux,
            extended_edges,
            relation_parameters,
            scaling_factor,
        );
        Self::accumulate_one_relation_evaluations::<LogDerivLookupRelation>(
            &mut univariate_accumulators.r_lookup,
            extended_edges,
            relation_parameters,
            scaling_factor,
        );
        Self::accumulate_one_relation_evaluations::<Poseidon2ExternalRelation>(
            &mut univariate_accumulators.r_pos_ext,
            extended_edges,
            relation_parameters,
            scaling_factor,
        );
        Self::accumulate_one_relation_evaluations::<Poseidon2InternalRelation>(
            &mut univariate_accumulators.r_pos_int,
            extended_edges,
            relation_parameters,
            scaling_factor,
        );
    }

    pub(crate) fn compute_full_relation_purported_value(
        purported_evaluations: &ClaimedEvaluations,
        relation_parameters: &RelationParameters,
        gate_sparators: GateSeparatorPolynomial,
    ) -> ScalarField {
        let mut relation_evaluations = AllRelationEvaluations::default();

        Self::accumulate_relation_evaluations(
            &mut relation_evaluations,
            purported_evaluations,
            relation_parameters,
            &gate_sparators.partial_evaluation_result,
        );

        let running_challenge = ScalarField::one();

        relation_evaluations
            .scale_and_batch_elements(running_challenge, &relation_parameters.alphas)
    }
}
