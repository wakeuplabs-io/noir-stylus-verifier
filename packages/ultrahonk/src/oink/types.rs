use crate::{
    types::{G1Affine, ScalarField, WitnessEntities},
    NUM_ALPHAS,
};

#[derive(Default)]
pub struct VerifierMemory {
    pub public_input_delta: ScalarField,
    pub witness_commitments: WitnessEntities<G1Affine>,
    pub challenges: Challenges,
}

pub struct Challenges {
    pub eta_1: ScalarField,
    pub eta_2: ScalarField,
    pub eta_3: ScalarField,
    pub beta: ScalarField,
    pub gamma: ScalarField,
    pub alphas: [ScalarField; NUM_ALPHAS],
}

impl Default for Challenges {
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
