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

#[cfg_attr(test, derive(Eq, PartialEq, Debug))]
pub struct Challenges {
    pub(crate) eta_1: ScalarField,
    pub(crate) eta_2: ScalarField,
    pub(crate) eta_3: ScalarField,
    pub(crate) beta: ScalarField,
    pub(crate) gamma: ScalarField,
    pub(crate) alphas: [ScalarField; NUM_ALPHAS],
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
