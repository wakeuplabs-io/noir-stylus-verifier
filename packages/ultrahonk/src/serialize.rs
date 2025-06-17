use crate::{constants::{NUM_BYTES_FELT, NUM_U64S_FELT}, types::{G1Affine, G1BaseField, G2Affine, G2BaseField, MontFp256}};
use alloc::vec::Vec;
use ark_ec::AffineRepr;
use ark_ff::{BigInteger, Field, MontConfig, PrimeField, Zero};
use num_bigint::BigUint;

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

impl<P: MontConfig<NUM_U64S_FELT>> BytesSerializable for MontFp256<P> {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        self.into_bigint().to_bytes_be()
    }
}

impl<P: MontConfig<NUM_U64S_FELT>> BytesDeserializable for MontFp256<P> {
    const SER_LEN: usize = NUM_BYTES_FELT;

    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, SerdeError> {
        const NUM_64_LIMBS: u32 = G1BaseField::MODULUS_BIT_SIZE.div_ceil(64);
        let mut fields = Vec::with_capacity(G1BaseField::extension_degree() as usize);

        let mut offset = 0;
        for _ in 0..G1BaseField::extension_degree() {
            let mut bigint: BigUint = Default::default();
            for _ in 0..NUM_64_LIMBS {
                let data = u64::deserialize_from_bytes_with_offset(&bytes, &mut offset).unwrap();
                bigint <<= 64;
                bigint += data;
            }
            fields.push(MontFp256::<P>::from(bigint));
        }

        Ok(MontFp256::<P>::from_base_prime_field_elems(fields).expect("Should work"))
    }

    fn deserialize_from_bytes_with_offset(
        bytes: &[u8],
        offset: &mut usize,
    ) -> Result<Self, SerdeError> {
        const NUM_64_LIMBS: u32 = G1BaseField::MODULUS_BIT_SIZE.div_ceil(64);
        let mut fields = Vec::with_capacity(G1BaseField::extension_degree() as usize);

        for _ in 0..G1BaseField::extension_degree() {
            let mut bigint: BigUint = Default::default();
            for _ in 0..NUM_64_LIMBS {
                let data = u64::deserialize_from_bytes_with_offset(&bytes, offset).unwrap();
                bigint <<= 64;
                bigint += data;
            }
            fields.push(MontFp256::<P>::from(bigint));
        }

        Ok(MontFp256::<P>::from_base_prime_field_elems(fields).expect("Should work"))
    }
}

impl<P: MontConfig<NUM_U64S_FELT>> BytesSerializable for Vec<MontFp256<P>> {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        let num_64_limbs: u32 = <MontFp256<P> as PrimeField>::MODULUS_BIT_SIZE.div_ceil(64);
        let fieldsize_bytes: u32 = num_64_limbs * 8;
        let field_size = fieldsize_bytes as usize * MontFp256::<P>::extension_degree() as usize;

        let total_size = self.len() as u32 * field_size as u32;

        let mut res = Vec::with_capacity(total_size as usize);
        for el in self.iter().cloned() {
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
        let zero = G1BaseField::zero();
        let (x, y) = self.xy().unwrap_or((zero, zero));
        let mut bytes = Vec::with_capacity(NUM_BYTES_FELT * 2);
        bytes.extend(x.serialize_to_bytes());
        bytes.extend(y.serialize_to_bytes());
        bytes

        // const NUM_64_LIMBS: u32 = G1BaseField::MODULUS_BIT_SIZE.div_ceil(64);
        // const FIELDSIZE_BYTES: u32 = NUM_64_LIMBS * 8;
        // const GROUPSIZE_BYTES: u32 = FIELDSIZE_BYTES * 2; // Times extension degree

        // let mut res = Vec::new();

        // if self.is_zero() {
        //     for _ in 0..GROUPSIZE_BYTES {
        //         res.push(255);
        //     }
        // } else {
        //     let (x, y) = self.xy().unwrap_or_default();
        //     res.extend(x.serialize_to_bytes());
        //     res.extend(y.serialize_to_bytes());
        // }

        // res
    }
}

impl BytesDeserializable for G1Affine {
    const SER_LEN: usize = 64;

    fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, SerdeError> {
        if bytes.iter().all(|&x| x == 255) {
            return Ok(G1Affine::zero());
        }

        let mut offset = 0;
        let first = G1BaseField::deserialize_from_bytes_with_offset(bytes, &mut offset).unwrap();
        let second = G1BaseField::deserialize_from_bytes_with_offset(bytes, &mut offset).unwrap();

        // read x first every time
        Ok(G1Affine::new(first, second))
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
