use ark_bn254::G1Affine;

use crate::{decider::types::{ClaimedEvaluations, VerifierCommitments}, types::{PrecomputedEntities, ScalarField, ShiftedWitnessEntities, WitnessEntities}};

pub struct PolyF<'a, T: Default> {
    pub precomputed: &'a PrecomputedEntities<T>,
    pub witness: &'a WitnessEntities<T>,
}

impl<'a> From<&'a VerifierCommitments> for PolyF<'a, G1Affine> {
    fn from(verifier_commitments: &'a VerifierCommitments) -> Self {
        Self {
            precomputed: &verifier_commitments.precomputed,
            witness: &verifier_commitments.witness,
        }
    }
}

impl<T: Default> PolyF<'_, T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.precomputed.iter().chain(self.witness.iter())
    }
}

impl<'a> From<&'a ClaimedEvaluations> for PolyF<'a, ScalarField> {
    fn from(claimed_evals: &'a ClaimedEvaluations) -> Self {
        Self {
            precomputed: &claimed_evals.precomputed,
            witness: &claimed_evals.witness,
        }
    }
}

pub struct PolyG<'a, T: Default> {
    pub wires: &'a [T; 5],
}

impl<T: Default> PolyG<'_, T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.wires.iter()
    }
}

impl<'a> From<&'a VerifierCommitments> for PolyG<'a, G1Affine> {

    fn from(verifier_commitments: &'a VerifierCommitments) -> Self {
        Self {
            wires: verifier_commitments.witness.to_be_shifted().try_into().unwrap(),
        }
    }
}

pub struct PolyGShift<'a, T: Default> {
    pub wires: &'a ShiftedWitnessEntities<T>,
}

impl<'a> From<&'a ClaimedEvaluations> for PolyGShift<'a, ScalarField> {
    fn from(claimed_evals: &'a ClaimedEvaluations) -> Self {
        Self {
            wires: &claimed_evals.shifted_witness,
        }
    }
}

impl<T: Default> PolyGShift<'_, T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.wires.iter()
    }
}
