// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use stylus_sdk::{alloy_primitives::U256, prelude::*};

#[derive(Clone, Default, Encode, Decode, BorshSize)]
pub struct Proposal {
    description: String,
    deadline: U256,
    for_votes: U256,
    against_votes: U256,
}


#[external]
pub trait UltraVerifier {
    fn verify(&self, proof: Bytes, input: Bytes) -> bool;
}

#[entrypoint]
pub struct Voting {
    merkle_root: StorageValue<B256>,
    proposal_count: StorageValue<U256>,
    proposals: StorageMap<U256, Proposal>,
    nullifiers: StorageMap<B256, bool>,
    verifier: StorageValue<Address>,
}

#[public]
impl Voting {

    #[constructor]
    pub fn new(merkle_root: B256, verifier: Address) -> Self {
        Self {
            merkle_root: StorageValue::new(merkle_root),
            verifier: StorageValue::new(verifier),
            proposal_count: StorageValue::new(U256::ZERO),
            proposals: StorageMap::new(),
            nullifiers: StorageMap::new(),
        }
    }

    pub fn propose(&mut self, description: String, deadline: U256) -> U256 {
        let proposal_id = *self.proposal_count.get();
        let proposal = Proposal {
            description,
            deadline,
            for_votes: U256::ZERO,
            against_votes: U256::ZERO,
        };
        self.proposals.insert(proposal_id, proposal);
        self.proposal_count.set(proposal_id + U256::from(1));
        proposal_id
    }

    pub fn cast_vote(
        &mut self,
        proof: Vec<B256>,
        proposal_id: U256,
        vote: U256,
        nullifier_hash: B256,
    ) -> Result<bool, Vec<u8>> {
        if self.nullifiers.get(nullifier_hash).unwrap_or(false) {
            return Err(b"Proof already submitted".to_vec());
        }

        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(b"Proposal not found".to_vec())?;

        if evm::block_timestamp() >= proposal.deadline {
            return Err(b"Voting period is over".to_vec());
        }

        self.nullifiers.insert(nullifier_hash, true);

        let leaf_input = stylus_sdk::keccak256(&stylus_sdk::keccak256(&abi::encode(&(
            self.merkle_root.get(),
            B256::from_word(proposal_id),
            B256::from_word(vote),
            nullifier_hash,
        ))));

        let verifier_addr = *self.verifier.get();
        let verifier = UltraVerifier::new(verifier_addr);

        if !verifier.verify(proof.clone(), leaf_input) {
            return Err(b"Invalid proof".to_vec());
        }

        if vote == U256::from(1) {
            proposal.for_votes += U256::from(1);
        } else {
            proposal.against_votes += U256::from(1);
        }

        Ok(true)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_counter() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let mut contract = Verifier::from(&vm);

        assert_eq!(U256::ZERO, contract.number());

        contract.increment();
        assert_eq!(U256::from(1), contract.number());

        contract.add_number(U256::from(3));
        assert_eq!(U256::from(4), contract.number());

        contract.mul_number(U256::from(2));
        assert_eq!(U256::from(8), contract.number());

        contract.set_number(U256::from(100));
        assert_eq!(U256::from(100), contract.number());

        // Override the msg value for future contract method invocations.
        vm.set_value(U256::from(2));

        contract.add_from_msg_value();
        assert_eq!(U256::from(102), contract.number());
    }
}
