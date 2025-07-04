use super::types::VerifierMemory;
use crate::backends::G1ArithmeticBackend;
use crate::backends::HashBackend;
use core::marker::PhantomData;

pub struct DeciderVerifier<P: G1ArithmeticBackend, H: HashBackend> {
    pub memory: VerifierMemory,
    phantom_data: PhantomData<P>,
    phantom_hasher: PhantomData<H>,
}

impl<P: G1ArithmeticBackend, H: HashBackend> DeciderVerifier<P, H> {
    pub fn new(memory: VerifierMemory) -> Self {
        Self {
            memory,
            phantom_data: PhantomData,
            phantom_hasher: PhantomData,
        }
    }
}
