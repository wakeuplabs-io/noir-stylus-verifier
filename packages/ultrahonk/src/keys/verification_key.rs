use crate::constants::PRECOMPUTED_ENTITIES_SIZE;
use crate::serialize::BytesDeserializable;
use crate::types::{
    G1Affine, G1BaseField, G2Affine, HonkProofError, HonkProofResult, PrecomputedEntities,
};
use ark_ff::PrimeField;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct VerifyingKey {
    pub crs: G2Affine,
    pub circuit_size: u32,
    pub num_public_inputs: u32,
    pub pub_inputs_offset: u32,
    pub pairing_inputs_public_input_key: PublicComponentKey,
    pub commitments: PrecomputedEntities<G1Affine>,
}

impl VerifyingKey {
    pub fn from_barrettenberg_and_crs(
        barretenberg_vk: VerifyingKeyBarretenberg,
        crs: G2Affine,
    ) -> Self {
        Self {
            crs,
            circuit_size: barretenberg_vk.circuit_size as u32,
            num_public_inputs: barretenberg_vk.num_public_inputs as u32,
            pub_inputs_offset: barretenberg_vk.pub_inputs_offset as u32,
            commitments: barretenberg_vk.commitments,
            pairing_inputs_public_input_key: barretenberg_vk.pairing_inputs_public_input_key,
        }
    }
}

pub struct VerifyingKeyBarretenberg {
    pub circuit_size: u64,
    pub log_circuit_size: u64,
    pub num_public_inputs: u64,
    pub pub_inputs_offset: u64,
    pub pairing_inputs_public_input_key: PublicComponentKey,
    pub commitments: PrecomputedEntities<G1Affine>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PublicComponentKey {
    start_idx: u32,
}

impl Default for PublicComponentKey {
    fn default() -> Self {
        Self {
            start_idx: u32::MAX,
        }
    }
}
impl PublicComponentKey {
    pub fn new(start_idx: u32) -> Self {
        Self { start_idx }
    }
    pub fn set(&mut self, start_idx: u32) {
        self.start_idx = start_idx;
    }
    pub fn is_set(&self) -> bool {
        self.start_idx != u32::MAX
    }
}

impl VerifyingKeyBarretenberg {
    const NUM_64_LIMBS: u32 = <G1BaseField as PrimeField>::MODULUS_BIT_SIZE.div_ceil(64);
    const FIELDSIZE_BYTES: u32 = Self::NUM_64_LIMBS * 8;
    const SER_FULL_SIZE: usize =
        4 * 8 + 4 + PRECOMPUTED_ENTITIES_SIZE * 2 * Self::FIELDSIZE_BYTES as usize;
    const SER_COMPRESSED_SIZE: usize = Self::SER_FULL_SIZE - 4;

    pub fn from_buffer(buf: &[u8]) -> HonkProofResult<Self> {
        let size = buf.len();
        if size != Self::SER_FULL_SIZE && size != Self::SER_COMPRESSED_SIZE {
            return Err(HonkProofError::InvalidKeyLength);
        }

        // Read data
        let mut offset = 0;
        let circuit_size = u64::deserialize_from_bytes_with_offset(&buf, &mut offset).unwrap();
        let log_circuit_size = u64::deserialize_from_bytes_with_offset(&buf, &mut offset).unwrap();
        let num_public_inputs = u64::deserialize_from_bytes_with_offset(&buf, &mut offset).unwrap();
        let pub_inputs_offset = u64::deserialize_from_bytes_with_offset(&buf, &mut offset).unwrap();
        let pairing_inputs_public_input_key = if size == Self::SER_FULL_SIZE {
            PublicComponentKey {
                start_idx: u32::deserialize_from_bytes_with_offset(&buf, &mut offset).unwrap(),
            }
        } else {
            Default::default()
        };

        let mut commitments = PrecomputedEntities::default();

        for el in commitments.iter_mut() {
            *el = G1Affine::deserialize_from_bytes_with_offset(buf, &mut offset).unwrap();
        }

        debug_assert!(offset == Self::SER_FULL_SIZE || offset == Self::SER_COMPRESSED_SIZE);

        Ok(Self {
            circuit_size,
            log_circuit_size,
            num_public_inputs,
            pub_inputs_offset,
            commitments,
            pairing_inputs_public_input_key,
        })
    }
}
