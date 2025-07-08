use crate::{
    constants::{NUM_BYTES_FELT, NUM_U64S_FELT},
    decider::types::{ClaimedEvaluations, RelationParameters, VerifierCommitments, VerifierMemory},
    transcript::{RoundData, Transcript},
    types::{AllEntities, G1Affine, G1BaseField, G2Affine, G2BaseField, MontFp256, ScalarField},
    NUM_ALPHAS,
};
use alloc::{string::String, vec::Vec};
use ark_ec::AffineRepr;
use ark_ff::{BigInteger, Field, MontConfig, PrimeField, Zero};

/// An error that occurs during de/serialization
#[derive(Debug)]
pub enum SerdeError {
    /// A sequence of deserialized elements is not the expected length
    InvalidLength,
    /// An error in the conversion of a type into a BN254 scalar field element
    ScalarConversion,
}

/// A trait for serializing types into byte arrays
pub trait BytesSerializable {
    /// Serializes a type into a vector of bytes,
    /// for use in precompiles or the transcript
    fn serialize_to_bytes(&self) -> Vec<u8>;
}

/// A trait for deserializing types from byte arrays
pub trait BytesDeserializable {
    /// The number of bytes expected to be deserialized
    const SER_LEN: usize;

    /// Deserializes a type from a slice of bytes,
    /// returned from a precompile or transcript operation
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, SerdeError>
    where
        Self: Sized;

    /// Deserializes a type from a slice of bytes at the given offset,
    /// returned from a precompile or transcript operation
    fn deserialize_from_bytes_with_offset(
        bytes: &[u8],
        offset: &mut usize,
    ) -> Result<Self, SerdeError>
    where
        Self: Sized,
    {
        let res = Self::deserialize_from_bytes(&bytes[*offset..*offset + Self::SER_LEN])?;
        *offset += Self::SER_LEN;
        Ok(res)
    }
}

impl BytesSerializable for u32 {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl BytesDeserializable for u32 {
    const SER_LEN: usize = 4;

    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, SerdeError> {
        Ok(u32::from_be_bytes(
            bytes.try_into().map_err(|_| SerdeError::InvalidLength)?,
        ))
    }
}

impl BytesSerializable for u64 {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl BytesDeserializable for u64 {
    const SER_LEN: usize = 8;

    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, SerdeError> {
        Ok(u64::from_be_bytes(
            bytes.try_into().map_err(|_| SerdeError::InvalidLength)?,
        ))
    }
}

impl BytesSerializable for bool {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

impl BytesDeserializable for bool {
    const SER_LEN: usize = 1;

    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, SerdeError> {
        Ok(bytes[0] != 0)
    }
}

impl<P: MontConfig<NUM_U64S_FELT>> BytesSerializable for MontFp256<P> {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        self.into_bigint().to_bytes_be()
    }
}

impl<P: MontConfig<NUM_U64S_FELT>> BytesDeserializable for MontFp256<P> {
    const SER_LEN: usize = NUM_BYTES_FELT;

    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, SerdeError> {
        // Number of 64-bit limbs needed to represent a base-field element
        const NUM_64_LIMBS: u32 = G1BaseField::MODULUS_BIT_SIZE.div_ceil(64);
        let fieldsize_bytes: usize = (NUM_64_LIMBS * 8) as usize;
        let ext_degree = G1BaseField::extension_degree() as usize;
        let expected_len = fieldsize_bytes * ext_degree;
        if bytes.len() != expected_len {
            return Err(SerdeError::InvalidLength);
        }

        // Reconstruct each coefficient directly from its BE byte slice.
        let mut offset = 0;
        let mut coeffs = Vec::with_capacity(ext_degree);
        for _ in 0..ext_degree {
            let slice = &bytes[offset..offset + fieldsize_bytes];
            coeffs.push(MontFp256::<P>::from_be_bytes_mod_order(slice));
            offset += fieldsize_bytes;
        }

        Ok(MontFp256::<P>::from_base_prime_field_elems(coeffs).expect("Should work"))
    }

    fn deserialize_from_bytes_with_offset(
        bytes: &[u8],
        offset: &mut usize,
    ) -> Result<Self, SerdeError> {
        // Compute the byte length of one field element (including extension degree)
        const NUM_64_LIMBS: u32 = G1BaseField::MODULUS_BIT_SIZE.div_ceil(64);
        let fieldsize_bytes: usize = (NUM_64_LIMBS * 8) as usize;
        let ext_degree = G1BaseField::extension_degree() as usize;
        let total_bytes = fieldsize_bytes * ext_degree;

        if *offset + total_bytes > bytes.len() {
            return Err(SerdeError::InvalidLength);
        }

        let res = Self::deserialize_from_bytes(&bytes[*offset..*offset + total_bytes])?;
        *offset += total_bytes;
        Ok(res)
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
        debug_assert_eq!(res.len(), total_size as usize);

        res
    }
}

impl<P: MontConfig<NUM_U64S_FELT>> BytesDeserializable for Vec<MontFp256<P>> {
    const SER_LEN: usize = 8;

    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, SerdeError> {
        let num_64_limbs: u32 = <MontFp256<P> as PrimeField>::MODULUS_BIT_SIZE.div_ceil(64);
        let fieldsize_bytes: u32 = num_64_limbs * 8;

        let size = bytes.len();
        let mut offset = 0;

        // Check sizes

        let num_elements = size / fieldsize_bytes as usize;
        if num_elements * fieldsize_bytes as usize != size {
            return Err(SerdeError::InvalidLength);
        }

        // Read data
        let mut res = Vec::with_capacity(num_elements);
        for _ in 0..num_elements {
            res.push(
                MontFp256::<P>::deserialize_from_bytes_with_offset(bytes, &mut offset).unwrap(),
            );
        }
        debug_assert_eq!(offset, size);

        Ok(res)
    }
}

impl BytesSerializable for G1Affine {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        // Use 0xFF… as the canonical infinity encoding expected by the
        // deserialiser elsewhere in the codebase.
        if self.infinity {
            return vec![0xFFu8; NUM_BYTES_FELT * 2];
        }

        let (x, y) = self.xy().expect("non-infinity point must have coordinates");
        let mut bytes = Vec::with_capacity(NUM_BYTES_FELT * 2);
        bytes.extend(x.serialize_to_bytes());
        bytes.extend(y.serialize_to_bytes());
        bytes
    }
}

impl BytesDeserializable for G1Affine {
    const SER_LEN: usize = 64;

    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, SerdeError> {
        let mut cursor = 0;
        let x = G1BaseField::deserialize_from_bytes_with_offset(bytes, &mut cursor).unwrap();
        let y = G1BaseField::deserialize_from_bytes_with_offset(bytes, &mut cursor).unwrap();

        Ok(G1Affine {
            x,
            y,
            infinity: x.is_zero() && y.is_zero(),
        })
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
    const SER_LEN: usize = NUM_BYTES_FELT * 4;

    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, SerdeError> {
        let mut cursor = 0;
        let x_c1 = G1BaseField::deserialize_from_bytes_with_offset(bytes, &mut cursor)?;
        let x_c0 = G1BaseField::deserialize_from_bytes_with_offset(bytes, &mut cursor)?;
        let y_c1 = G1BaseField::deserialize_from_bytes_with_offset(bytes, &mut cursor)?;
        let y_c0 = G1BaseField::deserialize_from_bytes_with_offset(bytes, &mut cursor)?;

        let x = G2BaseField { c0: x_c0, c1: x_c1 };
        let y = G2BaseField { c0: y_c0, c1: y_c1 };

        Ok(G2Affine {
            x,
            y,
            infinity: x.is_zero() && y.is_zero(),
        })
    }
}

impl BytesSerializable for VerifierCommitments {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Serialize witness commitments
        bytes.extend(self.witness.iter().flat_map(|x| x.serialize_to_bytes()));
        // Serialize precomputed commitments
        bytes.extend(self.precomputed.iter().flat_map(|x| x.serialize_to_bytes()));
        // Serialize shifted witness commitments
        bytes.extend(
            self.shifted_witness
                .iter()
                .flat_map(|x| x.serialize_to_bytes()),
        );

        bytes
    }
}

impl BytesDeserializable for VerifierCommitments {
    const SER_LEN: usize = 2560;

    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, SerdeError> {
        let mut offset = 0;
        let mut commitments = AllEntities::default();

        // panic!("bytes: {:?}", commitments.serialize_to_bytes().len());

        // Deserialize witness commitments
        for commitment in commitments.witness.iter_mut() {
            *commitment = G1Affine::deserialize_from_bytes_with_offset(bytes, &mut offset)?;
        }
        // Deserialize precomputed commitments
        for commitment in commitments.precomputed.iter_mut() {
            *commitment = G1Affine::deserialize_from_bytes_with_offset(bytes, &mut offset)?;
        }
        // Deserialize shifted witness commitments
        for commitment in commitments.shifted_witness.iter_mut() {
            *commitment = G1Affine::deserialize_from_bytes_with_offset(bytes, &mut offset)?;
        }

        // panic!("VerifierCommitments offset: {:?}", offset);

        Ok(AllEntities {
            witness: commitments.witness,
            precomputed: commitments.precomputed,
            shifted_witness: commitments.shifted_witness,
        })
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
    const SER_LEN: usize = 1892;

    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, SerdeError> {
        let mut offset = 0;

        // Deserialize scalar field elements
        let eta_1 = ScalarField::deserialize_from_bytes_with_offset(bytes, &mut offset)?;
        let eta_2 = ScalarField::deserialize_from_bytes_with_offset(bytes, &mut offset)?;
        let eta_3 = ScalarField::deserialize_from_bytes_with_offset(bytes, &mut offset)?;
        let beta = ScalarField::deserialize_from_bytes_with_offset(bytes, &mut offset)?;
        let gamma = ScalarField::deserialize_from_bytes_with_offset(bytes, &mut offset)?;
        let public_input_delta =
            ScalarField::deserialize_from_bytes_with_offset(bytes, &mut offset)?;

        // Deserialize fixed-size alphas array
        let mut alphas = [ScalarField::default(); NUM_ALPHAS];
        for alpha in alphas.iter_mut() {
            *alpha = ScalarField::deserialize_from_bytes_with_offset(bytes, &mut offset)?;
        }

        // Deserialize dynamic-size gate_challenges
        let challenges_len = u32::deserialize_from_bytes_with_offset(bytes, &mut offset)? as usize;
        let mut gate_challenges = Vec::with_capacity(challenges_len);
        for _ in 0..challenges_len {
            gate_challenges.push(ScalarField::deserialize_from_bytes_with_offset(
                bytes,
                &mut offset,
            )?);
        }

        // panic!("RelationParameters offset: {:?}", offset);

        Ok(RelationParameters {
            eta_1,
            eta_2,
            eta_3,
            beta,
            gamma,
            public_input_delta,
            alphas,
            gate_challenges,
        })
    }
}

impl BytesSerializable for ClaimedEvaluations {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Serialize witness evaluations
        bytes.extend(self.witness.iter().flat_map(|x| x.serialize_to_bytes()));
        // Serialize precomputed evaluations
        bytes.extend(self.precomputed.iter().flat_map(|x| x.serialize_to_bytes()));
        // Serialize shifted witness evaluations
        bytes.extend(
            self.shifted_witness
                .iter()
                .flat_map(|x| x.serialize_to_bytes()),
        );

        bytes
    }
}

impl BytesDeserializable for ClaimedEvaluations {
    const SER_LEN: usize = 1280;

    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, SerdeError> {
        let mut offset = 0;
        let mut evaluations = AllEntities::default();

        // Deserialize witness evaluations
        for eval in evaluations.witness.iter_mut() {
            *eval = ScalarField::deserialize_from_bytes_with_offset(bytes, &mut offset)?;
        }
        // Deserialize precomputed evaluations
        for eval in evaluations.precomputed.iter_mut() {
            *eval = ScalarField::deserialize_from_bytes_with_offset(bytes, &mut offset)?;
        }
        // Deserialize shifted witness evaluations
        for eval in evaluations.shifted_witness.iter_mut() {
            *eval = ScalarField::deserialize_from_bytes_with_offset(bytes, &mut offset)?;
        }

        Ok(evaluations)
    }
}

impl BytesSerializable for VerifierMemory {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Serialize each component using their implementations
        bytes.extend(self.verifier_commitments.serialize_to_bytes());
        bytes.extend(self.relation_parameters.serialize_to_bytes());
        bytes.extend(self.claimed_evaluations.serialize_to_bytes());

        bytes
    }
}

impl BytesDeserializable for VerifierMemory {
    const SER_LEN: usize =
        VerifierCommitments::SER_LEN + RelationParameters::SER_LEN + ClaimedEvaluations::SER_LEN;

    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, SerdeError> {
        let mut offset = 0;

        // Deserialize each component using their implementations
        let verifier_commitments =
            VerifierCommitments::deserialize_from_bytes_with_offset(bytes, &mut offset)?;
        let relation_parameters =
            RelationParameters::deserialize_from_bytes_with_offset(bytes, &mut offset)?;
        let claimed_evaluations =
            ClaimedEvaluations::deserialize_from_bytes_with_offset(bytes, &mut offset)?;

        Ok(VerifierMemory {
            verifier_commitments,
            relation_parameters,
            claimed_evaluations,
        })
    }
}

impl BytesSerializable for RoundData {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Serialize challenge_label vector
        bytes.extend((self.challenge_label.len() as u32).serialize_to_bytes());
        for label in &self.challenge_label {
            let label_bytes = label.as_bytes();
            bytes.extend((label_bytes.len() as u32).serialize_to_bytes());
            bytes.extend(label_bytes);
        }

        // Serialize entries vector
        bytes.extend((self.entries.len() as u32).serialize_to_bytes());
        for (label, size) in &self.entries {
            let label_bytes = label.as_bytes();
            bytes.extend((label_bytes.len() as u32).serialize_to_bytes());
            bytes.extend(label_bytes);
            bytes.extend((*size as u32).serialize_to_bytes());
        }

        bytes
    }
}

impl BytesDeserializable for RoundData {
    const SER_LEN: usize = 0; // Dynamic size

    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, SerdeError> {
        let mut offset = 0;

        // TODO: should remove?
        // Deserialize challenge_label vector
        let challenge_label_len =
            u32::deserialize_from_bytes_with_offset(bytes, &mut offset)? as usize;
        let mut challenge_label = Vec::with_capacity(challenge_label_len);
        for _ in 0..challenge_label_len {
            let label_len = u32::deserialize_from_bytes_with_offset(bytes, &mut offset)? as usize;
            if offset + label_len > bytes.len() {
                return Err(SerdeError::InvalidLength);
            }
            let label_bytes = &bytes[offset..offset + label_len];
            let label = String::from_utf8(label_bytes.to_vec())
                .map_err(|_| SerdeError::ScalarConversion)?;
            challenge_label.push(label);
            offset += label_len;
        }

        // Deserialize entries vector
        let entries_len = u32::deserialize_from_bytes_with_offset(bytes, &mut offset)? as usize;
        let mut entries = Vec::with_capacity(entries_len);
        for _ in 0..entries_len {
            let label_len = u32::deserialize_from_bytes_with_offset(bytes, &mut offset)? as usize;
            if offset + label_len > bytes.len() {
                return Err(SerdeError::InvalidLength);
            }
            let label_bytes = &bytes[offset..offset + label_len];
            let label = String::from_utf8(label_bytes.to_vec())
                .map_err(|_| SerdeError::ScalarConversion)?;
            offset += label_len;
            let size = u32::deserialize_from_bytes_with_offset(bytes, &mut offset)? as usize;
            entries.push((label, size));
        }

        Ok(RoundData {
            challenge_label,
            entries,
        })
    }

    fn deserialize_from_bytes_with_offset(
        bytes: &[u8],
        offset: &mut usize,
    ) -> Result<Self, SerdeError> {
        let start_offset = *offset;
        let result = Self::deserialize_from_bytes(&bytes[start_offset..])?;

        // Calculate how many bytes were consumed by re-serializing and measuring
        let consumed_bytes = result.serialize_to_bytes().len();
        *offset += consumed_bytes;

        Ok(result)
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
    const SER_LEN: usize = 0; // Dynamic size with prefixes

    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, SerdeError> {
        let mut offset = 0;

        // Deserialize proof_data with size prefix
        let proof_data_size = u32::deserialize_from_bytes_with_offset(bytes, &mut offset)? as usize;
        if offset + proof_data_size > bytes.len() {
            return Err(SerdeError::InvalidLength);
        }
        let proof_data =
            Vec::<ScalarField>::deserialize_from_bytes(&bytes[offset..offset + proof_data_size])?;
        offset += proof_data_size;

        // Deserialize counters and state (fixed sizes)
        let num_frs_written = u32::deserialize_from_bytes_with_offset(bytes, &mut offset)? as usize;
        let num_frs_read = u32::deserialize_from_bytes_with_offset(bytes, &mut offset)? as usize;
        let round_number = u32::deserialize_from_bytes_with_offset(bytes, &mut offset)? as usize;
        if offset >= bytes.len() {
            return Err(SerdeError::InvalidLength);
        }
        let is_first_challenge_byte = bytes[offset];
        offset += 1;
        let is_first_challenge = is_first_challenge_byte != 0;

        // Deserialize current_round_data with size prefix
        let current_round_data_size =
            u32::deserialize_from_bytes_with_offset(bytes, &mut offset)? as usize;
        if offset + current_round_data_size > bytes.len() {
            return Err(SerdeError::InvalidLength);
        }
        let current_round_data = Vec::<ScalarField>::deserialize_from_bytes(
            &bytes[offset..offset + current_round_data_size],
        )?;
        offset += current_round_data_size;

        // Deserialize previous_challenge (fixed size)
        let previous_challenge =
            ScalarField::deserialize_from_bytes_with_offset(bytes, &mut offset)?;

        Ok(Transcript {
            proof_data,
            num_frs_written,
            num_frs_read,
            round_number,
            is_first_challenge,
            current_round_data,
            previous_challenge,
        })
    }
}
