use alloc::vec::Vec;
use stylus_sdk::crypto::keccak;
use ultrahonk::{serialize::Serialize, types::ScalarField};

/// The hashing backend used in the Stylus VM,
/// which uses the VM-accelerated Keccak-256 implementation
pub struct StylusHasher;

impl ultrahonk::prelude::HashBackend for StylusHasher {
    fn hash(buffer: Vec<ScalarField>) -> ScalarField {
        // Losing 2 bits of this is not an issue -> we can just reduce mod p
        let vec = Serialize::to_buffer(&buffer, false);
        let bytes = keccak(&vec);
        let hash_result = bytes.as_ref(); 

        let mut offset = 0;
        Serialize::read_field_element(hash_result, &mut offset)
    }
}
