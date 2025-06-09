use crate::{types::{G1Affine, ScalarField, WitnessEntities}, NUM_ALPHAS};
use ark_ff::PrimeField;

pub(crate) struct VerifierMemory {
    pub(crate) public_input_delta: ScalarField,
    pub(crate) witness_commitments: WitnessEntities<G1Affine>,
    pub(crate) challenges: Challenges<ScalarField>,
}

pub(crate) struct Challenges<F: PrimeField> {
    pub(crate) eta_1: F,
    pub(crate) eta_2: F,
    pub(crate) eta_3: F,
    pub(crate) beta: F,
    pub(crate) gamma: F,
    pub(crate) alphas: [F; NUM_ALPHAS],
}

impl<F: PrimeField> Default for Challenges<F> {
    fn default() -> Self {
        Self {
            eta_1: Default::default(),
            eta_2: Default::default(),
            eta_3: Default::default(),
            beta: Default::default(),
            gamma: Default::default(),
            alphas: [Default::default(); NUM_ALPHAS],
        }
    }
}

impl Default for VerifierMemory {
    fn default() -> Self {
        Self {
            public_input_delta: Default::default(),
            witness_commitments: Default::default(),
            challenges: Default::default(),
        }
    }
}
