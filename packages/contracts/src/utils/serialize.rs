//! Custom de/serialization logic used to:
//! 1. de/serialize objects to/from byte arrays for use in EVM precompiles &
//!    transcript operations
//! 2. serialize objects to scalar arrays for use as public proof inputs

use alloc::{vec, vec::Vec};
use ark_ec::AffineRepr;
use ark_ff::{BigInt, BigInteger, Fp256, MontBackend, MontConfig, PrimeField, Zero};
use ultrahonk::types::{G1Affine, G1BaseField, G2Affine, G2BaseField, ScalarField};
use super::constants::{NUM_BYTES_FELT, NUM_U64S_FELT, NUM_BYTES_U64};

/// Type alias for a 256-bit prime field element in Montgomery form
pub type MontFp256<P> = Fp256<MontBackend<P, NUM_U64S_FELT>>;


/// An error that occurs during de/serialization
#[derive(Debug)]
pub enum SerdeError {
    /// A sequence of deserialized elements is not the expected length
    InvalidLength,
    /// An error in the conversion of a type into a BN254 scalar field element
    ScalarConversion,
}

// -------------------------------
// | BYTE SERDE TRAIT DEFINITION |
// -------------------------------

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
}

// -------------------------
// | TRAIT IMPLEMENTATIONS |
// -------------------------

impl BytesSerializable for bool {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
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

impl<P: MontConfig<NUM_U64S_FELT>> BytesSerializable for MontFp256<P> {
    /// Serializes a field element into a big-endian byte array
    fn serialize_to_bytes(&self) -> Vec<u8> {
        self.into_bigint().to_bytes_be()
    }
}

impl<P: MontConfig<NUM_U64S_FELT>> BytesDeserializable for MontFp256<P> {
    const SER_LEN: usize = NUM_BYTES_FELT;

    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, SerdeError> {
        // Field elements are serialized as big-endian, so we need to reverse here
        // for `bigint_from_le_bytes`
        let mut bytes = bytes.to_vec();
        bytes.reverse();
        let bigint = bigint_from_le_bytes(&bytes)?;
        Self::from_bigint(bigint).ok_or(SerdeError::ScalarConversion)
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
    const SER_LEN: usize = NUM_BYTES_FELT * 2;

    /// Deserializes a G1 point from a byte array.
    ///
    /// This matches the format returned by the EVM `ecAdd` and `ecMul`
    /// precompiles, as specified here:
    /// https://eips.ethereum.org/EIPS/eip-196#encoding
    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, SerdeError> {
        // Note: although this performs modular reduction, it's safe to do so
        // since we can assume that precompiles will always correctly return
        // elements contained in the field
        let mut cursor = 0;
        let x = deserialize_cursor(bytes, &mut cursor)?;
        let y = deserialize_cursor(bytes, &mut cursor)?;

        Ok(G1Affine {
            x,
            y,
            infinity: x.is_zero() && y.is_zero(),
        })
    }
}

impl BytesSerializable for G2BaseField {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(NUM_BYTES_FELT * 2);
        bytes.extend(self.c0.serialize_to_bytes());
        bytes.extend(self.c1.serialize_to_bytes());
        bytes
    }
}

impl BytesDeserializable for G2BaseField {
    const SER_LEN: usize = NUM_BYTES_FELT * 2;

    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, SerdeError> {
        let mut cursor = 0;
        let c0 = deserialize_cursor(bytes, &mut cursor)?;
        let c1 = deserialize_cursor(bytes, &mut cursor)?;
        Ok(Self { c0, c1 })
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
    /// specified here: https://eips.ethereum.org/EIPS/eip-197#encoding
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
        let x_c1 = deserialize_cursor(bytes, &mut cursor)?;
        let x_c0 = deserialize_cursor(bytes, &mut cursor)?;
        let y_c1 = deserialize_cursor(bytes, &mut cursor)?;
        let y_c0 = deserialize_cursor(bytes, &mut cursor)?;

        let x = G2BaseField { c0: x_c0, c1: x_c1 };
        let y = G2BaseField { c0: y_c0, c1: y_c1 };

        Ok(G2Affine {
            x,
            y,
            infinity: x.is_zero() && y.is_zero(),
        })
    }
}

// ---------------------------------
// | SCALAR SERDE TRAIT DEFINITION |
// ---------------------------------

/// A trait for serializing types into arrays of scalars
pub trait ScalarSerializable {
    /// Serializes a type into a vector of scalars
    fn serialize_to_scalars(&self) -> Result<Vec<ScalarField>, SerdeError>;
}

// -----------
// | HELPERS |
// -----------

/// Deserializes a type from a slice of bytes starting at the cursor position,
/// and increments the cursor by the number of bytes deserialized.
fn deserialize_cursor<D: BytesDeserializable>(
    bytes: &[u8],
    cursor: &mut usize,
) -> Result<D, SerdeError> {
    let elem = D::deserialize_from_bytes(&bytes[*cursor..*cursor + D::SER_LEN])?;
    *cursor += D::SER_LEN;
    Ok(elem)
}

/// Converts a little-endian byte array into a [`BigInt`]
pub fn bigint_from_le_bytes(bytes: &[u8]) -> Result<BigInt<NUM_U64S_FELT>, SerdeError> {
    // This will right-pad the bytes with zero-bytes if the length is less than 8 *
    // NUM_BYTES_U64
    let mut bytes_to_convert = [0_u8; NUM_BYTES_FELT];
    bytes_to_convert[..bytes.len()].copy_from_slice(bytes);

    let mut u64s = [0u64; NUM_U64S_FELT];
    for i in 0..NUM_U64S_FELT {
        u64s[i] = u64::from_le_bytes(
            bytes_to_convert[i * NUM_BYTES_U64..(i + 1) * NUM_BYTES_U64]
                .try_into()
                // Unwrapping here is safe because we index by the exact number of bytes
                // in a u64
                .unwrap(),
        );
    }
    Ok(BigInt::<NUM_U64S_FELT>(u64s))
}
