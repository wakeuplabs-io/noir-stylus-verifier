use crate::{
    constants::{NUM_BYTES_FELT, NUM_U64S_FELT},
    decider::{
        sumcheck::verifier::SumcheckVerifierMemory,
        types::{ClaimedEvaluations, RelationParameters, VerifierCommitments},
    },
    oink::types::Challenges,
    transcript::Transcript,
    types::{
        AllEntities, G1Affine, G1BaseField, G2Affine, G2BaseField, HonkProof, MontFp256,
        PrecomputedEntities, ScalarField, VerifyingKey, WitnessEntities,
    },
    NUM_ALPHAS,
};
use alloc::vec::Vec;
use ark_ec::AffineRepr;
use ark_ff::{BigInteger, Field, MontConfig, PrimeField, Zero};

/// A trait for serializing types into byte arrays
pub trait BytesSerializable {
    /// Serializes a type into a vector of bytes,
    fn serialize_to_bytes(&self) -> Vec<u8>;
}

/// A trait for deserializing types from byte arrays
pub trait BytesDeserializable {
    /// Deserializes a type from a slice of bytes,
    /// returning the deserialized value and the number of bytes read
    #[allow(clippy::result_unit_err)]
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<(Self, usize), ()>
    where
        Self: Sized;
}

impl BytesSerializable for bool {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

impl BytesDeserializable for bool {
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<(Self, usize), ()> {
        Ok((bytes[0] != 0, 1))
    }
}

impl BytesSerializable for u32 {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl BytesDeserializable for u32 {
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<(Self, usize), ()> {
        Ok((
            u32::from_be_bytes(bytes[..4].try_into().map_err(|_| ())?),
            4,
        ))
    }
}

impl BytesSerializable for u64 {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl BytesDeserializable for u64 {
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<(Self, usize), ()> {
        Ok((
            u64::from_be_bytes(bytes[..8].try_into().map_err(|_| ())?),
            8,
        ))
    }
}

impl<P: MontConfig<NUM_U64S_FELT>> BytesSerializable for MontFp256<P> {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        self.into_bigint().to_bytes_be()
    }
}

impl<P: MontConfig<NUM_U64S_FELT>> BytesDeserializable for MontFp256<P> {
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<(Self, usize), ()> {
        // Number of 64-bit limbs needed to represent a base-field element
        const NUM_64_LIMBS: u32 = G1BaseField::MODULUS_BIT_SIZE.div_ceil(64);
        let fieldsize_bytes: usize = (NUM_64_LIMBS * 8) as usize;
        let ext_degree = G1BaseField::extension_degree() as usize;

        // Reconstruct each coefficient directly from its BE byte slice.
        let mut offset = 0;
        let mut coeffs = Vec::with_capacity(ext_degree);
        for _ in 0..ext_degree {
            let slice = &bytes[offset..offset + fieldsize_bytes];
            coeffs.push(MontFp256::<P>::from_be_bytes_mod_order(slice));
            offset += fieldsize_bytes;
        }

        Ok((
            MontFp256::<P>::from_base_prime_field_elems(coeffs).ok_or(())?,
            offset,
        ))
    }
}

impl<P: MontConfig<NUM_U64S_FELT>> BytesSerializable for Vec<MontFp256<P>> {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        let num_64_limbs: u32 = <MontFp256<P> as PrimeField>::MODULUS_BIT_SIZE.div_ceil(64);
        let fieldsize_bytes: u32 = num_64_limbs * 8;
        let field_size = fieldsize_bytes as usize * MontFp256::<P>::extension_degree() as usize;
        let total_size = self.len() as u32 * field_size as u32;

        let mut res = Vec::with_capacity(total_size as usize);
        for el in self.iter() {
            res.extend(el.serialize_to_bytes());
        }

        res
    }
}

impl<P: MontConfig<NUM_U64S_FELT>> BytesDeserializable for Vec<MontFp256<P>> {
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<(Self, usize), ()> {
        let num_64_limbs: usize =
            <MontFp256<P> as PrimeField>::MODULUS_BIT_SIZE.div_ceil(64) as usize;
        let fieldsize_bytes: usize = num_64_limbs * 8;

        // Check sizes
        let num_elements = bytes.len() / fieldsize_bytes;
        if num_elements * fieldsize_bytes != bytes.len() {
            return Err(());
        }

        let mut offset: usize = 0;
        let mut res = Vec::with_capacity(num_elements);
        for _ in 0..num_elements {
            let (val, size) =
                MontFp256::<P>::deserialize_from_bytes(&bytes[offset..offset + fieldsize_bytes])
                    .unwrap();
            offset += size;
            res.push(val);
        }

        Ok((res, bytes.len()))
    }
}

impl BytesSerializable for G1Affine {
    /// Serializes a G1 point into a big-endian byte array of its coordinates.
    ///
    /// This matches the format expected by the EVM `ecAdd`, `ecMul`, and
    /// `ecPairing` precompiles as specified here:
    /// https://eips.ethereum.org/EIPS/eip-197#encoding
    fn serialize_to_bytes(&self) -> Vec<u8> {
        let zero = G1BaseField::zero();
        let (x, y) = self.xy().unwrap_or((zero, zero));
        let mut bytes = Vec::with_capacity(NUM_BYTES_FELT * 2);
        bytes.extend(x.serialize_to_bytes());
        bytes.extend(y.serialize_to_bytes());
        bytes
    }
}

impl BytesDeserializable for G1Affine {
    /// Deserializes a G1 point from a byte array.
    ///
    /// This matches the format returned by the EVM `ecAdd` and `ecMul`
    /// precompiles, as specified here:
    /// https://eips.ethereum.org/EIPS/eip-196#encoding
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut cursor = 0;

        let x = deserialize_cursor::<G1BaseField>(bytes, &mut cursor)?;
        let y = deserialize_cursor::<G1BaseField>(bytes, &mut cursor)?;

        Ok((
            G1Affine {
                x,
                y,
                infinity: x.is_zero() && y.is_zero(),
            },
            cursor,
        ))
    }
}

impl BytesSerializable for Vec<G1Affine> {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        self.iter().flat_map(|x| x.serialize_to_bytes()).collect()
    }
}

impl BytesDeserializable for Vec<G1Affine> {
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut cursor = 0;
        let mut res = Vec::with_capacity(bytes.len() / NUM_BYTES_FELT);
        while cursor < bytes.len() {
            let (elem, size) = G1Affine::deserialize_from_bytes(&bytes[cursor..])?;
            res.push(elem);
            cursor += size;
        }

        Ok((res, cursor))
    }
}

impl BytesSerializable for G2Affine {
    /// Serializes a G2 point into a big-endian byte array of the coefficients
    /// of its coordinates in the extension field, i.e.:
    ///
    /// Given an element of the field extension F_p^2[i] represented as ai + b,
    /// where a and b are elements of F_p, its serialization is the
    /// concatenation of a and b in big-endian order.
    ///
    /// This matches the format expected by the EVM `ecPairing` precompile, as
    /// specified here: <https://eips.ethereum.org/EIPS/eip-197#encoding>
    fn serialize_to_bytes(&self) -> Vec<u8> {
        let zero = G2BaseField::zero();
        let (x, y) = self.xy().unwrap_or((zero, zero));
        let mut bytes = Vec::with_capacity(NUM_BYTES_FELT * 4);
        bytes.extend(x.c1.serialize_to_bytes());
        bytes.extend(x.c0.serialize_to_bytes());
        bytes.extend(y.c1.serialize_to_bytes());
        bytes.extend(y.c0.serialize_to_bytes());
        bytes
    }
}

impl BytesDeserializable for G2Affine {
    /// Serializes a G2 point into a big-endian byte array of the coefficients
    /// of its coordinates in the extension field, i.e.:
    ///
    /// Given an element of the field extension F_p^2[i] represented as ai + b,
    /// where a and b are elements of F_p, its serialization is the
    /// concatenation of a and b in big-endian order.
    ///
    /// This matches the format expected by the EVM `ecPairing` precompile, as
    /// specified here: https://eips.ethereum.org/EIPS/eip-197#encoding
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut cursor = 0;
        let x_c1 = deserialize_cursor::<G1BaseField>(bytes, &mut cursor)?;
        let x_c0 = deserialize_cursor::<G1BaseField>(bytes, &mut cursor)?;
        let y_c1 = deserialize_cursor::<G1BaseField>(bytes, &mut cursor)?;
        let y_c0 = deserialize_cursor::<G1BaseField>(bytes, &mut cursor)?;

        let x = G2BaseField { c0: x_c0, c1: x_c1 };
        let y = G2BaseField { c0: y_c0, c1: y_c1 };

        Ok((
            G2Affine {
                x,
                y,
                infinity: x.is_zero() && y.is_zero(),
            },
            cursor,
        ))
    }
}

impl BytesSerializable for VerifierCommitments {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend(self.witness.iter().flat_map(|x| x.serialize_to_bytes()));
        bytes.extend(self.precomputed.iter().flat_map(|x| x.serialize_to_bytes()));
        bytes.extend(
            self.shifted_witness
                .iter()
                .flat_map(|x| x.serialize_to_bytes()),
        );

        bytes
    }
}

impl BytesDeserializable for VerifierCommitments {
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut cursor = 0;
        let mut commitments = AllEntities::default();

        // Deserialize witness commitments
        for commitment in commitments.witness.iter_mut() {
            *commitment = deserialize_cursor::<G1Affine>(bytes, &mut cursor)?;
        }

        // Deserialize precomputed commitments
        for commitment in commitments.precomputed.iter_mut() {
            *commitment = deserialize_cursor::<G1Affine>(bytes, &mut cursor)?;
        }

        // Deserialize shifted witness commitments
        for commitment in commitments.shifted_witness.iter_mut() {
            *commitment = deserialize_cursor::<G1Affine>(bytes, &mut cursor)?;
        }

        Ok((
            AllEntities {
                witness: commitments.witness,
                precomputed: commitments.precomputed,
                shifted_witness: commitments.shifted_witness,
            },
            cursor,
        ))
    }
}

impl BytesSerializable for RelationParameters {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Serialize scalar field elements
        bytes.extend(self.eta_1.serialize_to_bytes());
        bytes.extend(self.eta_2.serialize_to_bytes());
        bytes.extend(self.eta_3.serialize_to_bytes());
        bytes.extend(self.beta.serialize_to_bytes());
        bytes.extend(self.gamma.serialize_to_bytes());
        bytes.extend(self.public_input_delta.serialize_to_bytes());

        // Serialize fixed-size alphas array
        for alpha in &self.alphas {
            bytes.extend(alpha.serialize_to_bytes());
        }

        // Serialize dynamic-size gate_challenges
        bytes.extend((self.gate_challenges.len() as u32).serialize_to_bytes());
        bytes.extend(
            self.gate_challenges
                .iter()
                .flat_map(|x| x.serialize_to_bytes()),
        );

        bytes
    }
}

impl BytesDeserializable for RelationParameters {
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut offset = 0;

        // Deserialize scalar field elements
        let eta_1 = deserialize_cursor::<ScalarField>(bytes, &mut offset)?;
        let eta_2 = deserialize_cursor::<ScalarField>(bytes, &mut offset)?;
        let eta_3 = deserialize_cursor::<ScalarField>(bytes, &mut offset)?;
        let beta = deserialize_cursor::<ScalarField>(bytes, &mut offset)?;
        let gamma = deserialize_cursor::<ScalarField>(bytes, &mut offset)?;
        let public_input_delta = deserialize_cursor::<ScalarField>(bytes, &mut offset)?;

        // Deserialize fixed-size alphas array
        let mut alphas = [ScalarField::default(); NUM_ALPHAS];
        for alpha in alphas.iter_mut() {
            *alpha = deserialize_cursor::<ScalarField>(bytes, &mut offset)?;
        }

        // Deserialize dynamic-size gate_challenges
        let challenges_len = deserialize_cursor::<u32>(bytes, &mut offset)?;

        let mut gate_challenges = Vec::with_capacity(challenges_len as usize);
        for _ in 0..challenges_len {
            gate_challenges.push(deserialize_cursor::<ScalarField>(bytes, &mut offset)?);
        }

        Ok((
            RelationParameters {
                eta_1,
                eta_2,
                eta_3,
                beta,
                gamma,
                public_input_delta,
                alphas,
                gate_challenges,
            },
            offset,
        ))
    }
}

impl BytesSerializable for ClaimedEvaluations {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend(self.witness.iter().flat_map(|x| x.serialize_to_bytes()));
        bytes.extend(self.precomputed.iter().flat_map(|x| x.serialize_to_bytes()));
        bytes.extend(
            self.shifted_witness
                .iter()
                .flat_map(|x| x.serialize_to_bytes()),
        );

        bytes
    }
}

impl BytesDeserializable for ClaimedEvaluations {
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut offset = 0;
        let mut evaluations = AllEntities::default();

        for eval in evaluations.witness.iter_mut() {
            *eval = deserialize_cursor::<ScalarField>(bytes, &mut offset)?;
        }

        for eval in evaluations.precomputed.iter_mut() {
            *eval = deserialize_cursor::<ScalarField>(bytes, &mut offset)?;
        }

        for eval in evaluations.shifted_witness.iter_mut() {
            *eval = deserialize_cursor::<ScalarField>(bytes, &mut offset)?;
        }

        Ok((evaluations, offset))
    }
}

impl BytesSerializable for WitnessEntities<G1Affine> {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        self.iter().flat_map(|x| x.serialize_to_bytes()).collect()
    }
}

impl BytesDeserializable for WitnessEntities<G1Affine> {
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut cursor = 0;
        let mut entities = WitnessEntities::default();

        for entity in entities.iter_mut() {
            *entity = deserialize_cursor::<G1Affine>(bytes, &mut cursor)?;
        }

        Ok((entities, cursor))
    }
}

impl BytesSerializable for PrecomputedEntities<G1Affine> {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        self.iter().flat_map(|x| x.serialize_to_bytes()).collect()
    }
}

impl BytesDeserializable for PrecomputedEntities<G1Affine> {
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut cursor = 0;
        let mut entities = PrecomputedEntities::default();

        for entity in entities.iter_mut() {
            *entity = deserialize_cursor::<G1Affine>(bytes, &mut cursor)?;
        }

        Ok((entities, cursor))
    }
}

impl BytesSerializable for SumcheckVerifierMemory {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.relation_parameters.serialize_to_bytes());
        bytes.extend(self.claimed_evaluations.serialize_to_bytes());
        bytes
    }
}

impl BytesDeserializable for SumcheckVerifierMemory {
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut offset = 0;
        let relation_parameters = deserialize_cursor::<RelationParameters>(bytes, &mut offset)?;
        let claimed_evaluations = deserialize_cursor::<ClaimedEvaluations>(bytes, &mut offset)?;
        Ok((
            Self {
                relation_parameters,
                claimed_evaluations,
            },
            offset,
        ))
    }
}

impl BytesSerializable for Challenges {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.eta_1.serialize_to_bytes());
        bytes.extend(self.eta_2.serialize_to_bytes());
        bytes.extend(self.eta_3.serialize_to_bytes());
        bytes.extend(self.beta.serialize_to_bytes());
        bytes.extend(self.gamma.serialize_to_bytes());
        for alpha in self.alphas {
            bytes.extend(alpha.serialize_to_bytes());
        }
        bytes
    }
}

impl BytesDeserializable for Challenges {
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut offset = 0;
        let eta_1 = deserialize_cursor::<ScalarField>(bytes, &mut offset)?;
        let eta_2 = deserialize_cursor::<ScalarField>(bytes, &mut offset)?;
        let eta_3 = deserialize_cursor::<ScalarField>(bytes, &mut offset)?;
        let beta = deserialize_cursor::<ScalarField>(bytes, &mut offset)?;
        let gamma = deserialize_cursor::<ScalarField>(bytes, &mut offset)?;
        let mut alphas = [ScalarField::zero(); NUM_ALPHAS];
        for alpha in alphas.iter_mut() {
            *alpha = deserialize_cursor::<ScalarField>(bytes, &mut offset)?;
        }

        Ok((
            Self {
                eta_1,
                eta_2,
                eta_3,
                beta,
                gamma,
                alphas,
            },
            offset,
        ))
    }
}

impl BytesSerializable for Transcript {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Serialize proof_data with size prefix
        let proof_data_bytes = self.proof_data.serialize_to_bytes();
        bytes.extend((proof_data_bytes.len() as u32).serialize_to_bytes());
        bytes.extend(proof_data_bytes);

        // Serialize counters and state (fixed sizes)
        bytes.extend((self.num_frs_written as u32).serialize_to_bytes());
        bytes.extend((self.num_frs_read as u32).serialize_to_bytes());
        bytes.extend((self.round_number as u32).serialize_to_bytes());
        bytes.push(if self.is_first_challenge { 1u8 } else { 0u8 });

        // Serialize current_round_data with size prefix
        let current_round_data_bytes = self.current_round_data.serialize_to_bytes();
        bytes.extend((current_round_data_bytes.len() as u32).serialize_to_bytes());
        bytes.extend(current_round_data_bytes);

        // Serialize previous_challenge (fixed size)
        bytes.extend(self.previous_challenge.serialize_to_bytes());

        bytes
    }
}

impl BytesDeserializable for Transcript {
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut offset = 0;

        // Deserialize proof_data with size prefix
        let proof_data_size = deserialize_cursor::<u32>(bytes, &mut offset)?;
        if offset + proof_data_size as usize > bytes.len() {
            return Err(());
        }
        let (proof_data, _) = Vec::<ScalarField>::deserialize_from_bytes(
            &bytes[offset..offset + proof_data_size as usize],
        )?;
        offset += proof_data_size as usize;

        // Deserialize counters and state (fixed sizes)
        let num_frs_written = deserialize_cursor::<u32>(bytes, &mut offset)?;
        let num_frs_read = deserialize_cursor::<u32>(bytes, &mut offset)?;
        let round_number = deserialize_cursor::<u32>(bytes, &mut offset)?;
        if offset >= bytes.len() {
            return Err(());
        }
        let is_first_challenge = deserialize_cursor::<bool>(bytes, &mut offset)?;

        // Deserialize current_round_data with size prefix
        let current_round_data_size = deserialize_cursor::<u32>(bytes, &mut offset)?;
        if offset + current_round_data_size as usize > bytes.len() {
            return Err(());
        }
        let current_round_data = Vec::<ScalarField>::deserialize_from_bytes(
            &bytes[offset..offset + current_round_data_size as usize],
        )?;
        offset += current_round_data_size as usize;

        // Deserialize previous_challenge (fixed size)
        let previous_challenge = deserialize_cursor::<ScalarField>(bytes, &mut offset)?;

        Ok((
            Transcript {
                proof_data,
                num_frs_written: num_frs_written as usize,
                num_frs_read: num_frs_read as usize,
                round_number: round_number as usize,
                is_first_challenge,
                current_round_data: current_round_data.0,
                previous_challenge,
            },
            offset,
        ))
    }
}

impl BytesDeserializable for VerifyingKey {
    fn deserialize_from_bytes(buf: &[u8]) -> Result<(Self, usize), ()> {
        let mut offset = 0;
        let circuit_size = deserialize_cursor::<u64>(buf, &mut offset)?;
        let _log_circuit_size = deserialize_cursor::<u64>(buf, &mut offset)?;
        let num_public_inputs = deserialize_cursor::<u64>(buf, &mut offset)?;
        let pub_inputs_offset = deserialize_cursor::<u64>(buf, &mut offset)?;

        let mut commitments = PrecomputedEntities::default();

        for el in commitments.iter_mut() {
            *el = deserialize_cursor::<G1Affine>(buf, &mut offset)?;
        }

        Ok((
            Self {
                circuit_size: circuit_size as u32,
                num_public_inputs: num_public_inputs as u32,
                pub_inputs_offset: pub_inputs_offset as u32,
                commitments,
            },
            offset,
        ))
    }
}

impl BytesDeserializable for HonkProof {
    fn deserialize_from_bytes(buf: &[u8]) -> Result<(Self, usize), ()> {
        let mut offset = 0;
        let proof = deserialize_cursor::<Vec<ScalarField>>(buf, &mut offset)?;
        Ok((Self::new(proof), offset))
    }
}

/// Deserializes a type from a slice of bytes starting at the cursor position,
/// and increments the cursor by the number of bytes deserialized.
fn deserialize_cursor<D: BytesDeserializable>(bytes: &[u8], cursor: &mut usize) -> Result<D, ()> {
    let (elem, size) = D::deserialize_from_bytes(&bytes[*cursor..])?;
    *cursor += size;
    Ok(elem)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::G1Affine;
    use ark_ff::{One, UniformRand};
    use rand::thread_rng;

    #[test]
    fn test_bool_serialization() {
        let test_cases = vec![true, false];

        for test_case in test_cases {
            let serialized = test_case.serialize_to_bytes();
            assert_eq!(serialized.len(), 1);

            let deserialized = bool::deserialize_from_bytes(&serialized).unwrap();
            assert_eq!(test_case, deserialized.0);
        }
    }

    #[test]
    fn test_u32_serialization() {
        let test_cases = vec![0u32, 1, 42, u32::MAX, u32::MIN];

        for test_case in test_cases {
            let serialized = test_case.serialize_to_bytes();
            assert_eq!(serialized.len(), 4);

            let deserialized = u32::deserialize_from_bytes(&serialized).unwrap();
            assert_eq!(test_case, deserialized.0);
        }
    }

    #[test]
    fn test_u64_serialization() {
        let test_cases = vec![0u64, 1, 42, u64::MAX, u64::MIN];

        for test_case in test_cases {
            let serialized = test_case.serialize_to_bytes();
            assert_eq!(serialized.len(), 8);

            let deserialized = u64::deserialize_from_bytes(&serialized).unwrap();
            assert_eq!(test_case, deserialized.0);
        }
    }

    #[test]
    fn test_montfp_scalarfield_serialization() {
        let mut rng = thread_rng();
        let test_cases = vec![
            ScalarField::zero(),
            ScalarField::one(),
            ScalarField::rand(&mut rng),
        ];

        for test_case in test_cases {
            let serialized = test_case.serialize_to_bytes();
            assert_eq!(serialized.len(), NUM_BYTES_FELT);

            let deserialized = ScalarField::deserialize_from_bytes(&serialized).unwrap();
            assert_eq!(test_case, deserialized.0);
        }
    }

    #[test]
    fn test_montfp_scalarfield_vec_serialization() {
        let test_cases = vec![
            vec![],
            vec![ScalarField::zero()],
            vec![ScalarField::one(), ScalarField::from(42u64)],
            vec![
                ScalarField::zero(),
                ScalarField::one(),
                ScalarField::from(42u64),
            ],
        ];

        for test_case in test_cases {
            let serialized = test_case.serialize_to_bytes();
            assert_eq!(serialized.len(), test_case.len() * NUM_BYTES_FELT);

            let deserialized = Vec::<ScalarField>::deserialize_from_bytes(&serialized).unwrap();
            assert_eq!(test_case, deserialized.0);
        }
    }

    #[test]
    fn test_g1_affine_serialization() {
        let mut rng = thread_rng();
        let test_cases = vec![
            G1Affine::generator(),
            G1Affine::identity(),
            G1Affine::rand(&mut rng),
        ];

        for test_case in test_cases {
            let serialized = test_case.serialize_to_bytes();
            let deserialized = G1Affine::deserialize_from_bytes(&serialized).unwrap();
            assert_eq!(test_case, deserialized.0);
        }
    }

    #[test]
    fn test_g1_affine_vec_serialization() {
        let test_cases = vec![vec![], vec![G1Affine::generator()]];

        for test_case in test_cases {
            let serialized = test_case.serialize_to_bytes();
            let deserialized = Vec::<G1Affine>::deserialize_from_bytes(&serialized).unwrap();
            assert_eq!(test_case, deserialized.0);
        }
    }

    #[test]
    fn test_g2_affine_serialization() {
        let mut rng = thread_rng();
        let test_cases = vec![
            G2Affine::generator(),
            G2Affine::identity(),
            G2Affine::rand(&mut rng),
        ];

        for test_case in test_cases {
            let serialized = test_case.serialize_to_bytes();
            let deserialized = G2Affine::deserialize_from_bytes(&serialized).unwrap();
            assert_eq!(test_case, deserialized.0);
        }
    }

    #[test]
    fn test_verifier_commitments_serialization() {
        let test_cases = vec![VerifierCommitments::default()];

        for test_case in test_cases {
            let serialized = test_case.serialize_to_bytes();
            let deserialized = VerifierCommitments::deserialize_from_bytes(&serialized).unwrap();
            assert_eq!(test_case, deserialized.0);
        }
    }

    #[test]
    fn test_relation_parameters_serialization() {
        let test_cases = vec![RelationParameters::default()];

        for test_case in test_cases {
            let serialized = test_case.serialize_to_bytes();
            let deserialized = RelationParameters::deserialize_from_bytes(&serialized).unwrap();
            assert_eq!(test_case, deserialized.0);
        }
    }

    #[test]
    fn test_claimed_evaluations_serialization() {
        let test_cases = vec![ClaimedEvaluations::default()];

        for test_case in test_cases {
            let serialized = test_case.serialize_to_bytes();
            let deserialized = ClaimedEvaluations::deserialize_from_bytes(&serialized).unwrap();
            assert_eq!(test_case, deserialized.0);
        }
    }

    #[test]
    fn test_witness_entities_serialization() {
        let test_cases = vec![WitnessEntities::<G1Affine>::default()];

        for test_case in test_cases {
            let serialized = test_case.serialize_to_bytes();
            let deserialized =
                WitnessEntities::<G1Affine>::deserialize_from_bytes(&serialized).unwrap();
            assert_eq!(test_case, deserialized.0);
        }
    }

    #[test]
    fn test_precomputed_entities_serialization() {
        let test_cases = vec![PrecomputedEntities::<G1Affine>::default()];

        for test_case in test_cases {
            let serialized = test_case.serialize_to_bytes();
            let deserialized =
                PrecomputedEntities::<G1Affine>::deserialize_from_bytes(&serialized).unwrap();
            assert_eq!(test_case, deserialized.0);
        }
    }

    #[test]
    fn test_sumcheck_verifier_memory_serialization() {
        let test_cases = vec![SumcheckVerifierMemory::default()];

        for test_case in test_cases {
            let serialized = test_case.serialize_to_bytes();
            let deserialized = SumcheckVerifierMemory::deserialize_from_bytes(&serialized).unwrap();
            assert_eq!(test_case, deserialized.0);
        }
    }

    #[test]
    fn test_challenges_serialization() {
        let test_cases = vec![Challenges::default()];

        for test_case in test_cases {
            let serialized = test_case.serialize_to_bytes();
            let deserialized = Challenges::deserialize_from_bytes(&serialized).unwrap();
            assert_eq!(test_case, deserialized.0);
        }
    }

    #[test]
    fn test_transcript_serialization() {
        let test_cases = vec![Transcript::default()];

        for test_case in test_cases {
            let serialized = test_case.serialize_to_bytes();
            let deserialized = Transcript::deserialize_from_bytes(&serialized).unwrap();
            assert_eq!(test_case, deserialized.0);
        }
    }
}
