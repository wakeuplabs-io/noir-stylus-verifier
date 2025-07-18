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
        types::{ClaimedEvaluations, RelationParameters},
    },
    types::ScalarField,
};
use ark_ff::One;

pub(crate) type SumcheckRoundOutput<const SIZE: usize> = Univariate<SIZE>;

pub(crate) struct SumcheckVerifierRound;

impl SumcheckVerifierRound {
    /// Given the evaluations  \f$P_1(u_0,\ldots, u_{d-1}), \ldots, P_N(u_0,\ldots, u_{d-1}) \f$ of the
    /// ProverPolynomials at the challenge point \f$(u_0,\ldots, u_{d-1})\f$ stored in extended_edges, this
    /// method computes the evaluation of \f$ \tilde{F} \f$ taking these values as arguments.
    pub(crate) fn compute_full_relation_purported_value(
        extended_edges: &ClaimedEvaluations,
        relation_parameters: &RelationParameters,
        scaling_factor: &ScalarField,
    ) -> ScalarField {
        let mut relation_evaluations = AllRelationEvaluations::default();

        Self::accumulate_one_relation_evaluations::<UltraArithmeticRelation>(
            &mut relation_evaluations.r_arith,
            extended_edges,
            relation_parameters,
            scaling_factor,
        );
        Self::accumulate_one_relation_evaluations::<UltraPermutationRelation>(
            &mut relation_evaluations.r_perm,
            extended_edges,
            relation_parameters,
            scaling_factor,
        );
        Self::accumulate_one_relation_evaluations::<DeltaRangeConstraintRelation>(
            &mut relation_evaluations.r_delta,
            extended_edges,
            relation_parameters,
            scaling_factor,
        );
        Self::accumulate_elliptic_curve_relation_evaluations(
            &mut relation_evaluations.r_elliptic,
            extended_edges,
            relation_parameters,
            scaling_factor,
        );
        Self::accumulate_one_relation_evaluations::<AuxiliaryRelation>(
            &mut relation_evaluations.r_aux,
            extended_edges,
            relation_parameters,
            scaling_factor,
        );
        Self::accumulate_one_relation_evaluations::<LogDerivLookupRelation>(
            &mut relation_evaluations.r_lookup,
            extended_edges,
            relation_parameters,
            scaling_factor,
        );
        Self::accumulate_one_relation_evaluations::<Poseidon2ExternalRelation>(
            &mut relation_evaluations.r_pos_ext,
            extended_edges,
            relation_parameters,
            scaling_factor,
        );
        Self::accumulate_one_relation_evaluations::<Poseidon2InternalRelation>(
            &mut relation_evaluations.r_pos_int,
            extended_edges,
            relation_parameters,
            scaling_factor,
        );

        let running_challenge = ScalarField::one();

        relation_evaluations
            .scale_and_batch_elements(running_challenge, &relation_parameters.alphas)
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
}
