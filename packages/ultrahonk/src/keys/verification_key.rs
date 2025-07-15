use crate::types::{G1Affine, PrecomputedEntities};

pub struct VerifyingKey {
    pub circuit_size: u32,
    pub(crate) num_public_inputs: u32,
    pub(crate) pub_inputs_offset: u32,
    pub(crate) commitments: PrecomputedEntities<G1Affine>,
}
