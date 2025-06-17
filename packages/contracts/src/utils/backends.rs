// use core::panic;

// use alloc::vec::Vec;
// use num_traits::identities::One;
// use ultrahonk::{backends::G1ArithmeticError, serialize::{BytesSerializable, BytesDeserializable}, types::{G1Affine, G2Affine, ScalarField}};
// use stylus_sdk::{alloy_primitives::Address, call::RawCall, crypto::keccak};
// use crate::utils::constants::{EC_ADD_ADDRESS_LAST_BYTE, EC_MUL_ADDRESS_LAST_BYTE, EC_PAIRING_ADDRESS_LAST_BYTE, NUM_BYTES_FELT, PAIRING_CHECK_RESULT_LAST_BYTE_INDEX};

// /// The hashing backend used in the Stylus VM,
// /// which uses the VM-accelerated Keccak-256 implementation
// pub struct StylusHasher;

// impl ultrahonk::backends::HashBackend for StylusHasher {
//     fn hash(buffer: Vec<ScalarField>) -> ScalarField {
//         // Losing 2 bits of this is not an issue -> we can just reduce mod p
//         let vec = buffer.serialize_to_bytes();
//         let bytes = keccak(&vec);
//         let hash_result = bytes.as_ref(); 

//         let mut offset = 0;
//         ScalarField::deserialize_from_bytes_with_offset(hash_result, &mut offset).unwrap() // TODO: replace with deserialize_from_bytes once available
//     }
// }


// /// The G1 arithmetic backend used in the Stylus VM,
// /// which calls out to the EC arithmetic EVM precompiles
// pub struct PrecompileG1ArithmeticBackend;

// impl ultrahonk::backends::G1ArithmeticBackend for PrecompileG1ArithmeticBackend {
//     /// Calls the `ecAdd` precompile with the given points, handling
//     /// de/serialization
//     fn ec_add(a: G1Affine, b: G1Affine) -> Result<G1Affine, G1ArithmeticError> {
//         if a == G1Affine::identity() {
//             return Ok(b);
//         } else if b == G1Affine::identity() {
//             return Ok(a);
//         }

//         // Serialize the points
//         let mut calldata = [0_u8; NUM_BYTES_FELT * 4];
//         calldata[..NUM_BYTES_FELT * 2].copy_from_slice(&a.serialize_to_bytes());
//         calldata[NUM_BYTES_FELT * 2..].copy_from_slice(&b.serialize_to_bytes());

//         // cast call 0x0000000000000000000000000000000000000006 0x291d8296ce578d914fba64c4973ed3bea268984123f85d4af5a7eb97e82a99e5095c2aab35286b5a3f522cc201e0531d6ebf39b6fceed3b83d1afe36a37645260aaeafb577cdc2cad64b5e3e16c3e8e014340374be579e6489c116cc8f797afc113df6140dc8c4b92bd9ffcd4d3de194a8ee6bd132cab107c92b4f8a9d5b2f88 --rpc-url  https://sepolia-rollup.arbitrum.io/rpc
//         // panic!("cast call {:?} {:?} --rpc-url  https://sepolia-rollup.arbitrum.io/rpc", Address::with_last_byte(EC_ADD_ADDRESS_LAST_BYTE), hex::encode(calldata));

//         // Call the `ecAdd` precompile. TODO: fails here
//         let res_xy_bytes = unsafe {
//             RawCall::new_static()
//                 .call(Address::with_last_byte(EC_ADD_ADDRESS_LAST_BYTE), &calldata)
//                 .map_err(|_| G1ArithmeticError)?
//         };

//         // Deserialize the affine coordinates returned from the precompile
//         G1Affine::deserialize_from_bytes(&res_xy_bytes).map_err(|_| G1ArithmeticError)
//         // Ok(G1Affine::zero())
//     }

//     /// Calls the `ecMul` precompile with the given scalar and point, handling
//     /// de/serialization
//     fn ec_scalar_mul(a: ScalarField, b: G1Affine) -> Result<G1Affine, G1ArithmeticError> {
//         if a == ScalarField::one() {
//             return Ok(b);
//         }

//         // Serialize the point and scalar
//         let mut calldata = [0_u8; NUM_BYTES_FELT * 3];
//         calldata[..NUM_BYTES_FELT * 2].copy_from_slice(&b.serialize_to_bytes());
//         calldata[NUM_BYTES_FELT * 2..].copy_from_slice(&a.serialize_to_bytes());

//         // Call the `ecMul` precompile
//         let res_xy_bytes = unsafe {
//             RawCall::new_static()
//                 .call(Address::with_last_byte(EC_MUL_ADDRESS_LAST_BYTE), &calldata)
//                 .map_err(|_| G1ArithmeticError)?
//         };

//         // Deserialize the affine coordinates returned from the precompile
//         Ok(G1Affine::deserialize_from_bytes(res_xy_bytes.as_ref()).unwrap())
//     }

//     /// Calls the `ecPairing` precompile with the given points, handling
//     /// de/serialization
//     fn ec_pairing_check(
//         a_1: G1Affine,
//         a_2: G1Affine,
//         b_1: G2Affine,
//         b_2: G2Affine,
//     ) -> Result<bool, G1ArithmeticError> {
//         // Serialize the points
//         let mut calldata = [0_u8; NUM_BYTES_FELT * 12];
//         calldata[..NUM_BYTES_FELT * 2].copy_from_slice(&a_1.serialize_to_bytes());
//         calldata[NUM_BYTES_FELT * 2..NUM_BYTES_FELT * 6].copy_from_slice(&b_1.serialize_to_bytes());
//         calldata[NUM_BYTES_FELT * 6..NUM_BYTES_FELT * 8].copy_from_slice(&a_2.serialize_to_bytes());
//         calldata[NUM_BYTES_FELT * 8..].copy_from_slice(&b_2.serialize_to_bytes());

//         // Call the `ecPairing` precompile
//         let res = unsafe {
//             RawCall::new_static()
//                 // Only get the last byte of the 32-byte return data,
//                 // containing the boolean result
//             .limit_return_data(
//                 PAIRING_CHECK_RESULT_LAST_BYTE_INDEX, /* offset */
//                 1,                                    /* size */
//             )
//             .call(
//                 Address::with_last_byte(EC_PAIRING_ADDRESS_LAST_BYTE),
//                 &calldata,
//             )
//             .map_err(|_| G1ArithmeticError)?
//         };

//         // Return the result of the pairing check, which is either a 0 or 1.
//         Ok(res[0] == 1)
//     }
// }
