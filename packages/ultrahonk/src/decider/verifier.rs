use super::types::VerifierMemory;

pub struct DeciderVerifier {
    pub memory: VerifierMemory,
}

impl DeciderVerifier {
    pub fn new(memory: VerifierMemory) -> Self {
        Self { memory }
    }
}
