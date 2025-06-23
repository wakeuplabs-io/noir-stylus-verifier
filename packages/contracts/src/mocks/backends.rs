//! Testing contract which wraps EVM precompile functionality for testing
//! purposes. This contract is intended to be used in conjunction with a local
//! devnet

use crate::utils::{
    backends::{PrecompileG1ArithmeticBackend, PrecompileHashBackend},
    serde_def_types::{SerdeG1Affine, SerdeG2Affine, SerdeScalarField},
};
use alloc::vec::Vec;
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::backends::{G1ArithmeticBackend, HashBackend};

/// The precompile testing contract, which itself is stateless
#[storage]
#[cfg_attr(feature = "e2e-backends", entrypoint)]
struct PrecompileTestContract;

#[public]
impl PrecompileTestContract {
    /// Invokes the `ecAdd` precompile on the given inputs
    pub fn test_ec_add(&self, a_bytes: Bytes, b_bytes: Bytes) -> Result<Bytes, Vec<u8>> {
        let a: SerdeG1Affine = postcard::from_bytes(a_bytes.as_slice()).unwrap();
        let b: SerdeG1Affine = postcard::from_bytes(b_bytes.as_slice()).unwrap();

        let c = PrecompileG1ArithmeticBackend::ec_add(a.0, b.0).unwrap();
        let c_bytes = postcard::to_allocvec(&SerdeG1Affine(c)).unwrap();
        Ok(c_bytes.into())
    }

    /// Invokes the `ecMul` precompile on the given inputs
    pub fn test_ec_mul(&self, a_bytes: Bytes, b_bytes: Bytes) -> Result<Bytes, Vec<u8>> {
        let a: SerdeScalarField = postcard::from_bytes(a_bytes.as_slice()).unwrap();
        let b: SerdeG1Affine = postcard::from_bytes(b_bytes.as_slice()).unwrap();
        let c = PrecompileG1ArithmeticBackend::ec_scalar_mul(a.0, b.0).unwrap();
        Ok(postcard::to_allocvec(&SerdeG1Affine(c)).unwrap().into())
    }

    /// Invokes the `ecPairing` precompile on the given inputs
    pub fn test_ec_pairing(&self, a_bytes: Bytes, b_bytes: Bytes) -> Result<bool, Vec<u8>> {
        let a: SerdeG1Affine = postcard::from_bytes(a_bytes.as_slice()).unwrap();
        let b: SerdeG2Affine = postcard::from_bytes(b_bytes.as_slice()).unwrap();

        Ok(PrecompileG1ArithmeticBackend::ec_pairing_check(a.0, -a.0, b.0, b.0).unwrap())
    }

    /// Invokes the `hash` precompile on the given inputs
    pub fn test_hash(&self, a_bytes: Bytes) -> Result<Bytes, Vec<u8>> {
        let c = PrecompileHashBackend::hash(a_bytes.as_slice());
        Ok(c.to_vec().into())
    }
}
