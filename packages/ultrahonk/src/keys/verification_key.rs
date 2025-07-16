use crate::types::{G1Affine, PrecomputedEntities};

pub struct VerifyingKey {
    pub circuit_size: u32,
    pub commitments: PrecomputedEntities<G1Affine>,
    pub num_public_inputs: u32,
    pub pub_inputs_offset: u32,
}
