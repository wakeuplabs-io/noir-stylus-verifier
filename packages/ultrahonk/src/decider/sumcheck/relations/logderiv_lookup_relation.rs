use super::Relation;
use crate::alloc::borrow::ToOwned;
use crate::decider::types::{ClaimedEvaluations, RelationParameters};
use crate::types::ScalarField;

#[derive(Clone, Debug, Default)]
pub(crate) struct LogDerivLookupRelationEvals {
    pub(crate) r0: ScalarField,
    pub(crate) r1: ScalarField,
}

impl LogDerivLookupRelationEvals {
    pub(crate) fn scale_and_batch_elements(&self, running_challenge: &[ScalarField], result: &mut ScalarField) {
        assert!(running_challenge.len() == LogDerivLookupRelation::NUM_RELATIONS);

        *result += self.r0 * running_challenge[0];
        *result += self.r1 * running_challenge[1];
    }
}

pub(crate) struct LogDerivLookupRelation {}

impl LogDerivLookupRelation {
    pub(crate) const NUM_RELATIONS: usize = 2;
}

impl LogDerivLookupRelation {
    fn compute_inverse_exists_verifier(input: &ClaimedEvaluations) -> ScalarField {
        let row_has_write = input.witness.lookup_read_tags();
        let row_has_read = input.precomputed.q_lookup();

        -(row_has_write.to_owned() * row_has_read) + row_has_write + row_has_read
    }

    fn compute_read_term_verifier(
        input: &ClaimedEvaluations,
        relation_parameters: &RelationParameters,
    ) -> ScalarField {
        let gamma = &relation_parameters.gamma;
        let eta_1 = &relation_parameters.eta_1;
        let eta_2 = &relation_parameters.eta_2;
        let eta_3 = &relation_parameters.eta_3;
        let w_1 = input.witness.w_l();
        let w_2 = input.witness.w_r();
        let w_3 = input.witness.w_o();
        let w_1_shift = input.shifted_witness.w_l();
        let w_2_shift = input.shifted_witness.w_r();
        let w_3_shift = input.shifted_witness.w_o();
        let table_index = input.precomputed.q_o();
        let negative_column_1_step_size = input.precomputed.q_r();
        let negative_column_2_step_size = input.precomputed.q_m();
        let negative_column_3_step_size = input.precomputed.q_c();

        // The wire values for lookup gates are accumulators structured in such a way that the differences w_i -
        // step_size*w_i_shift result in values present in column i of a corresponding table. See the documentation in
        // method get_lookup_accumulators() in  for a detailed explanation.
        let derived_table_entry_1 =
            w_1.to_owned() + gamma + negative_column_1_step_size.to_owned() * w_1_shift;
        let derived_table_entry_2 = negative_column_2_step_size.to_owned() * w_2_shift + w_2;
        let derived_table_entry_3 = negative_column_3_step_size.to_owned() * w_3_shift + w_3;

        // (w_1 + \gamma q_2*w_1_shift) + η(w_2 + q_m*w_2_shift) + η₂(w_3 + q_c*w_3_shift) + η₃q_index.
        // deg 2 or 3
        derived_table_entry_1
            + derived_table_entry_2 * eta_1
            + derived_table_entry_3 * eta_2
            + table_index.to_owned() * eta_3
    }

    fn compute_write_term_verifier(
        input: &ClaimedEvaluations,
        relation_parameters: &RelationParameters,
    ) -> ScalarField {
        let gamma = &relation_parameters.gamma;
        let eta_1 = &relation_parameters.eta_1;
        let eta_2 = &relation_parameters.eta_2;
        let eta_3 = &relation_parameters.eta_3;

        let table_1 = input.precomputed.table_1();
        let table_2 = input.precomputed.table_2();
        let table_3 = input.precomputed.table_3();
        let table_4 = input.precomputed.table_4();

        table_1.to_owned()
            + gamma
            + table_2.to_owned() * eta_1
            + table_3.to_owned() * eta_2
            + table_4.to_owned() * eta_3
    }
}

impl Relation for LogDerivLookupRelation {
    type VerifyAcc = LogDerivLookupRelationEvals;

    fn verify_accumulate(
        univariate_accumulator: &mut Self::VerifyAcc,
        input: &ClaimedEvaluations,
        relation_parameters: &RelationParameters,
        scaling_factor: &ScalarField,
    ) {
        let inverses = input.witness.lookup_inverses(); // Degree 1
        let read_counts = input.witness.lookup_read_counts(); // Degree 1
        let read_selector = input.precomputed.q_lookup(); // Degree 1

        let inverse_exists = Self::compute_inverse_exists_verifier(input); // Degree 2
        let read_term = Self::compute_read_term_verifier(input, relation_parameters); // Degree 2 (3)
        let write_term = Self::compute_write_term_verifier(input, relation_parameters); // Degree 1 (2)
        let write_inverse = read_term.to_owned() * inverses; // Degree 3 (4)
        let read_inverse = write_term.to_owned() * inverses; // Degree 2 (3)

        // Establish the correctness of the polynomial of inverses I. Note: inverses is computed so that the value is 0
        // if !inverse_exists.
        // Degrees:                     2 (3)       1 (2)        1              1
        let tmp = (read_term * write_term * inverses - inverse_exists) * scaling_factor; // Deg 4 (6)
        univariate_accumulator.r0 += tmp;

        ///////////////////////////////////////////////////////////////////////

        // Establish validity of the read. Note: no scaling factor here since this constraint is 'linearly dependent,
        // i.e. enforced across the entire trace, not on a per-row basis.
        // Degrees:                       1            2 (3)            1            3 (4)
        let tmp = read_inverse * read_selector - write_inverse * read_counts; // Deg 4 (5)
        univariate_accumulator.r1 += tmp;
    }
}
