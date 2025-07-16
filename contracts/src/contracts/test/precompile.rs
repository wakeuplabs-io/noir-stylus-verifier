//! Testing contract which wraps EVM precompile functionality for testing
//! purposes.

use crate::utils::backends::{PrecompileG1ArithmeticBackend, PrecompileHashBackend};
use alloc::vec::Vec;
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::serialize::{BytesDeserializable, BytesSerializable};
use ultrahonk::{
    backends::{G1ArithmeticBackend, HashBackend},
    types::{G1Affine, G2Affine, ScalarField},
};

/// The precompile testing contract, which itself is stateless
#[storage]
#[entrypoint]
struct PrecompileTestContract;

#[public]
impl PrecompileTestContract {
    /// Invokes the `ecAdd` precompile on the given inputs
    pub fn test_ec_add(&self, a_bytes: Bytes, b_bytes: Bytes) -> Result<Bytes, Vec<u8>> {
        let a: G1Affine = G1Affine::deserialize_from_bytes(a_bytes.as_slice())
            .unwrap()
            .0;
        let b: G1Affine = G1Affine::deserialize_from_bytes(b_bytes.as_slice())
            .unwrap()
            .0;

        let c = PrecompileG1ArithmeticBackend::ec_add(a, b).unwrap();
        Ok(c.serialize_to_bytes().into())
    }

    /// Invokes the `ecMul` precompile on the given inputs
    pub fn test_ec_mul(&self, a_bytes: Bytes, b_bytes: Bytes) -> Result<Bytes, Vec<u8>> {
        let a: ScalarField = ScalarField::deserialize_from_bytes(a_bytes.as_slice())
            .unwrap()
            .0;
        let b: G1Affine = G1Affine::deserialize_from_bytes(b_bytes.as_slice())
            .unwrap()
            .0;
        let c = PrecompileG1ArithmeticBackend::ec_scalar_mul(a, b).unwrap();

        Ok(c.serialize_to_bytes().into())
    }

    /// Invokes the `ecPairing` precompile on the given inputs
    pub fn test_ec_pairing(&self, a_bytes: Bytes, b_bytes: Bytes) -> Result<bool, Vec<u8>> {
        let a: G1Affine = G1Affine::deserialize_from_bytes(a_bytes.as_slice())
            .unwrap()
            .0;
        let b: G2Affine = G2Affine::deserialize_from_bytes(b_bytes.as_slice())
            .unwrap()
            .0;

        Ok(PrecompileG1ArithmeticBackend::ec_pairing_check(a, -a, b, b).unwrap())
    }

    /// Invokes the `hash` precompile on the given inputs
    pub fn test_hash(&self, a_bytes: Bytes) -> Result<Bytes, Vec<u8>> {
        let c = PrecompileHashBackend::hash(a_bytes.as_slice());
        Ok(c.to_vec().into())
    }
}
