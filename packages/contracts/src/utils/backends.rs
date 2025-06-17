use crate::utils::constants::{
    EC_ADD_ADDRESS_LAST_BYTE, EC_MUL_ADDRESS_LAST_BYTE, EC_PAIRING_ADDRESS_LAST_BYTE,
    PAIRING_CHECK_RESULT_LAST_BYTE_INDEX,
};
use alloc::vec::Vec;
use num_traits::identities::One;
use stylus_sdk::prelude::*;
use stylus_sdk::{alloy_primitives::Address, call::RawCall, crypto::keccak};
use ultrahonk::{
    backends::G1ArithmeticError,
    constants::NUM_BYTES_FELT,
    serialize::{BytesDeserializable, BytesSerializable},
    types::{G1Affine, G2Affine, ScalarField},
};

/// The hashing backend used in the Stylus VM,
/// which uses the VM-accelerated Keccak-256 implementation
pub struct StylusHasher;

impl ultrahonk::backends::HashBackend for StylusHasher {
    fn hash(buffer: Vec<ScalarField>) -> ScalarField {
        // Losing 2 bits of this is not an issue -> we can just reduce mod p
        let vec = buffer.serialize_to_bytes();
        let bytes = keccak(&vec);
        let hash_result = bytes.as_ref();

        ScalarField::deserialize_from_bytes(hash_result).unwrap()
    }
}

/// The G1 arithmetic backend used in the Stylus VM,
/// which calls out to the EC arithmetic EVM precompiles
#[storage]
pub struct PrecompileG1ArithmeticBackend;

impl ultrahonk::backends::G1ArithmeticBackend for PrecompileG1ArithmeticBackend {
    /// Calls the `ecAdd` precompile with the given points, handling
    /// de/serialization
    fn ec_add(a: G1Affine, b: G1Affine) -> Result<G1Affine, G1ArithmeticError> {
        if a == G1Affine::identity() {
            return Ok(b);
        } else if b == G1Affine::identity() {
            return Ok(a);
        }

        // Serialize the points
        let mut calldata = [0_u8; NUM_BYTES_FELT * 4];
        calldata[..NUM_BYTES_FELT * 2].copy_from_slice(&a.serialize_to_bytes());
        calldata[NUM_BYTES_FELT * 2..].copy_from_slice(&b.serialize_to_bytes());

        // Call the `ecAdd` precompile.
        let res_xy_bytes = unsafe {
            RawCall::new_static()
                .call(Address::with_last_byte(EC_ADD_ADDRESS_LAST_BYTE), &calldata)
                .map_err(|_| G1ArithmeticError)?
        };

        // Deserialize the affine coordinates returned from the precompile
        G1Affine::deserialize_from_bytes(&res_xy_bytes).map_err(|_| G1ArithmeticError)
    }

    /// Calls the `ecMul` precompile with the given scalar and point, handling
    /// de/serialization
    fn ec_scalar_mul(a: ScalarField, b: G1Affine) -> Result<G1Affine, G1ArithmeticError> {
        if a == ScalarField::one() {
            return Ok(b);
        }

        // Serialize the point and scalar
        let mut calldata = [0_u8; NUM_BYTES_FELT * 3];
        calldata[..NUM_BYTES_FELT * 2].copy_from_slice(&b.serialize_to_bytes());
        calldata[NUM_BYTES_FELT * 2..].copy_from_slice(&a.serialize_to_bytes());

        // Call the `ecMul` precompile
        let res_xy_bytes = unsafe {
            RawCall::new_static()
                .call(Address::with_last_byte(EC_MUL_ADDRESS_LAST_BYTE), &calldata)
                .map_err(|_| G1ArithmeticError)?
        };

        // Deserialize the affine coordinates returned from the precompile
        Ok(G1Affine::deserialize_from_bytes(res_xy_bytes.as_ref()).unwrap())
    }

    /// Calls the `ecPairing` precompile with the given points, handling
    /// de/serialization
    fn ec_pairing_check(
        a_1: G1Affine,
        a_2: G1Affine,
        b_1: G2Affine,
        b_2: G2Affine,
    ) -> Result<bool, G1ArithmeticError> {
        // Serialize the points
        let mut calldata = [0_u8; NUM_BYTES_FELT * 12];
        calldata[..NUM_BYTES_FELT * 2].copy_from_slice(&a_1.serialize_to_bytes());
        calldata[NUM_BYTES_FELT * 2..NUM_BYTES_FELT * 6].copy_from_slice(&b_1.serialize_to_bytes());
        calldata[NUM_BYTES_FELT * 6..NUM_BYTES_FELT * 8].copy_from_slice(&a_2.serialize_to_bytes());
        calldata[NUM_BYTES_FELT * 8..].copy_from_slice(&b_2.serialize_to_bytes());

        // Call the `ecPairing` precompile
        let res = unsafe {
            RawCall::new_static()
                // Only get the last byte of the 32-byte return data,
                // containing the boolean result
                .limit_return_data(
                    PAIRING_CHECK_RESULT_LAST_BYTE_INDEX, /* offset */
                    1,                                    /* size */
                )
                .call(
                    Address::with_last_byte(EC_PAIRING_ADDRESS_LAST_BYTE),
                    &calldata,
                )
                .map_err(|_| G1ArithmeticError)?
        };

        // Return the result of the pairing check, which is either a 0 or 1.
        Ok(res[0] == 1)
    }
}
