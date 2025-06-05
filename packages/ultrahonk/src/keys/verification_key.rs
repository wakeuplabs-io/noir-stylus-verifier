use crate::polynomials::polynomial_types::PrecomputedEntities;
use crate::{
    honk_curve::HonkCurve,
    polynomials::polynomial_types::PRECOMPUTED_ENTITIES_SIZE,
    serialize::{Serialize, SerializeP},
    types::ScalarField,
    HonkProofError, HonkProofResult,
};
use ark_ec::pairing::Pairing;
use serde::{Deserialize, Serialize as SerdeSerialize};

#[derive(Clone)]
pub struct VerifyingKey<P: Pairing> {
    pub crs: P::G2Affine,
    pub circuit_size: u32,
    pub num_public_inputs: u32,
    pub pub_inputs_offset: u32,
    pub pairing_inputs_public_input_key: PublicComponentKey,
    pub commitments: PrecomputedEntities<P::G1Affine>,
}

impl<P: Pairing> VerifyingKey<P> {
    pub fn from_barrettenberg_and_crs(
        barretenberg_vk: VerifyingKeyBarretenberg<P>,
        crs: P::G2Affine,
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

pub struct VerifyingKeyBarretenberg<P: Pairing> {
    pub circuit_size: u64,
    pub log_circuit_size: u64,
    pub num_public_inputs: u64,
    pub pub_inputs_offset: u64,
    pub pairing_inputs_public_input_key: PublicComponentKey,
    pub commitments: PrecomputedEntities<P::G1Affine>,
}

#[derive(Clone, Copy, Debug, SerdeSerialize, Deserialize)]
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

impl<P: HonkCurve<ScalarField>> VerifyingKeyBarretenberg<P> {
    const FIELDSIZE_BYTES: u32 = SerializeP::<P>::FIELDSIZE_BYTES;
    const SER_FULL_SIZE: usize =
        4 * 8 + 4 + PRECOMPUTED_ENTITIES_SIZE * 2 * Self::FIELDSIZE_BYTES as usize;
    const SER_COMPRESSED_SIZE: usize = Self::SER_FULL_SIZE - 4;

    pub fn from_buffer(buf: &[u8]) -> HonkProofResult<Self> {
        let size = buf.len();
        let mut offset = 0;
        if size != Self::SER_FULL_SIZE && size != Self::SER_COMPRESSED_SIZE {
            return Err(HonkProofError::InvalidKeyLength);
        }

        // Read data
        let circuit_size = Serialize::<P::ScalarField>::read_u64(buf, &mut offset);
        let log_circuit_size = Serialize::<P::ScalarField>::read_u64(buf, &mut offset);
        let num_public_inputs = Serialize::<P::ScalarField>::read_u64(buf, &mut offset);
        let pub_inputs_offset = Serialize::<P::ScalarField>::read_u64(buf, &mut offset);
        let pairing_inputs_public_input_key = if size == Self::SER_FULL_SIZE {
            PublicComponentKey {
                start_idx: Serialize::<P::ScalarField>::read_u32(buf, &mut offset),
            }
        } else {
            Default::default()
        };

        let mut commitments = PrecomputedEntities::default();

        for el in commitments.iter_mut() {
            *el = SerializeP::<P>::read_g1_element(buf, &mut offset, true);
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
