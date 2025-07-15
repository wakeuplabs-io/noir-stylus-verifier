use crate::serialize::BytesDeserializable;
use crate::types::{
    G1Affine, G1BaseField, HonkProofError, HonkProofResult, PrecomputedEntities,
    PRECOMPUTED_ENTITIES_SIZE,
};
use ark_ff::PrimeField;

pub struct VerifyingKey {
    pub circuit_size: u32,
    pub(crate) num_public_inputs: u32,
    pub(crate) pub_inputs_offset: u32,
    pub(crate) commitments: PrecomputedEntities<G1Affine>,
}

// TODO: move to serialize.rs?
impl VerifyingKey {
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
        let (circuit_size, size) = u64::deserialize_from_bytes(&buf[offset..]).unwrap();
        offset += size;
        let (_log_circuit_size, size) = u64::deserialize_from_bytes(&buf[offset..]).unwrap();
        offset += size;
        let (num_public_inputs, size) = u64::deserialize_from_bytes(&buf[offset..]).unwrap();
        offset += size;
        let (pub_inputs_offset, size) = u64::deserialize_from_bytes(&buf[offset..]).unwrap();
        offset += size;

        let mut commitments = PrecomputedEntities::default();

        for el in commitments.iter_mut() {
            let (val, size) = G1Affine::deserialize_from_bytes(&buf[offset..]).unwrap();
            *el = val;
            offset += size;
        }

        debug_assert!(offset == Self::SER_FULL_SIZE || offset == Self::SER_COMPRESSED_SIZE);

        Ok(Self {
            circuit_size: circuit_size as u32,
            num_public_inputs: num_public_inputs as u32,
            pub_inputs_offset: pub_inputs_offset as u32,
            commitments,
        })
    }
}
