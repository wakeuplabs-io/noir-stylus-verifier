// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::{string::String, vec::Vec};
use alloy_primitives::Address;
use alloy_sol_types::sol;
use alloy_sol_types::SolCall;
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::U256,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageString, StorageU256},
    stylus_core::calls::context::Call,
};

sol! {
    function verify(bytes memory proof, bytes memory input) external view returns (bool);

    #[derive(Debug)]
    struct Proposal {
        string description;
        uint256 deadline;
        uint256 for_votes;
        uint256 against_votes;
        uint256 voters_root;
    }
}

#[storage]
struct StorageProposal {
    description: StorageString,
    deadline: StorageU256,
    for_votes: StorageU256,
    against_votes: StorageU256,
    started: StorageBool,
    voters_root: StorageU256,
}

#[storage]
#[entrypoint]
struct Voting {
    verifier: StorageAddress,
    proposal_count: StorageU256,
    proposals: StorageMap<U256, StorageProposal>,
    nullifiers: StorageMap<U256, StorageBool>,
}

#[public]
impl Voting {
    #[constructor]
    pub fn constructor(&mut self, verifier: Address) {
        self.verifier.set(verifier);
        self.proposal_count.set(U256::ZERO);
    }

    pub fn get_verifier(&self) -> Address {
        self.verifier.get()
    }

    pub fn get_proposal(&self, proposal_id: U256) -> Result<(String, U256, U256, U256, U256), Vec<u8>> {
        let proposal = self.proposals.get(proposal_id);
        if !proposal.started.get() {
            return Err(b"Proposal not found".to_vec());
        }

        Ok((
            proposal.description.get_string(),
            proposal.deadline.get(),
            proposal.for_votes.get(),
            proposal.against_votes.get(),
            proposal.voters_root.get(),
        ))
    }

    pub fn propose(&mut self, description: String, deadline: U256, voters_root: U256) -> U256 {
        let proposal_id = self.proposal_count.get();

        // store the proposal in the storage
        self.proposals
            .setter(proposal_id)
            .description
            .set_str(description);
        self.proposals.setter(proposal_id).deadline.set(deadline);
        self.proposals.setter(proposal_id).for_votes.set(U256::ZERO);
        self.proposals
            .setter(proposal_id)
            .against_votes
            .set(U256::ZERO);
        self.proposals.setter(proposal_id).started.set(true);
        self.proposals
            .setter(proposal_id)
            .voters_root
            .set(voters_root);

        // increment the proposal count and return the id
        self.proposal_count.set(proposal_id + U256::from(1));
        proposal_id
    }

    pub fn cast_vote(
        &mut self,
        proof: Bytes,
        proposal_id: U256,
        vote: U256,
        nullifier_hash: U256,
    ) -> Result<bool, Vec<u8>> {
        // Check if the proposal exists and is started
        let proposal = self.proposals.get(proposal_id);
        if !proposal.started.get() {
            return Err(b"Proposal not found".to_vec());
        }

        // Check if the voting period is over
        if U256::from(self.vm().block_timestamp()) >= proposal.deadline.get() {
            return Err(b"Voting period over".to_vec());
        }

        // Check if the nullifier hash has already been used
        if self.nullifiers.get(nullifier_hash) {
            return Err(b"Proof already submitted".to_vec());
        }
        self.nullifiers.insert(nullifier_hash, true);

        // verify the proof
        if !self.call_verify(
            proof,
            proposal.voters_root.get(),
            proposal_id,
            vote,
            nullifier_hash,
        ) {
            return Err(b"Invalid proof".to_vec());
        }

        // Update the vote counts
        let current_for_votes = proposal.for_votes.get();
        let current_against_votes = proposal.against_votes.get();
        if vote == U256::from(1) {
            self.proposals
                .setter(proposal_id)
                .for_votes
                .set(current_for_votes + U256::from(1));
        } else {
            self.proposals
                .setter(proposal_id)
                .against_votes
                .set(current_against_votes + U256::from(1));
        }

        Ok(true)
    }

    fn call_verify(
        &self,
        proof: Bytes,
        proposal_root: U256,
        proposal_id: U256,
        vote: U256,
        nullifier_hash: U256,
    ) -> bool {
        let calldata = verifyCall {
            proof: proof.to_vec().into(),
            input: Vec::from(
                [
                    proposal_root.to_be_bytes::<32>(),
                    nullifier_hash.to_be_bytes::<32>(),
                    proposal_id.to_be_bytes::<32>(),
                    vote.to_be_bytes::<32>(),
                ]
                .concat(),
            )
            .into(),
        }
        .abi_encode();

        let res = self
            .vm()
            .static_call(&Call::new(), self.verifier.get(), &calldata);

        if let Ok(output) = res {
            if let Ok(decoded) = verifyCall::abi_decode_returns(&output, true) {
                return decoded._0;
            }
        }

        false
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloy_primitives::address;
    use alloy_sol_types::SolCall;

    sol! {
        function verify(bytes memory proof, bytes memory input) external view returns (bool);
    }

    const MOCK_VERIFIER: Address = address!("0x0000000000000000000000000000000000000001");

    #[test]
    fn test_propose_returns_proposal_id() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let mut contract = Voting::from(&vm);

        let proposal_id =
            contract.propose("Test Proposal".to_string(), U256::from(1000), U256::from(1));
        assert_eq!(proposal_id, U256::from(0));
    }

    #[test]
    fn test_propose_stores_proposal() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let mut contract = Voting::from(&vm);

        let proposal_id =
            contract.propose("Test Proposal".to_string(), U256::from(1000), U256::from(1));
        let proposal = contract.proposals.get(proposal_id);

        assert_eq!(proposal.description.get_string(), "Test Proposal");
        assert_eq!(proposal.deadline.get(), U256::from(1000));
        assert_eq!(proposal.for_votes.get(), U256::ZERO);
        assert_eq!(proposal.against_votes.get(), U256::ZERO);
        assert_eq!(proposal.started.get(), true);
    }

    #[test]
    fn test_cast_vote_returns_true() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let mut contract = Voting::from(&vm);
        contract.verifier.set(MOCK_VERIFIER);
        let voters_root = U256::from(1);

        // propose a proposal
        let proposal_id =
            contract.propose("Test Proposal".to_string(), U256::from(1000), voters_root);

        // mock proofs and validation call
        let proof = vec![1u8, 2, 3, 4];
        let vote = U256::from(1);
        let nullifier_hash = U256::from(5);

        // build calldata for mock
        let calldata = verifyCall {
            proof: proof.clone().into(),
            input: Vec::from(
                [
                    voters_root.to_be_bytes::<32>(),
                    nullifier_hash.to_be_bytes::<32>(),
                    proposal_id.to_be_bytes::<32>(),
                    vote.to_be_bytes::<32>(),
                ]
                .concat(),
            )
            .into(),
        }
        .abi_encode();
        let return_data = verifyCall::abi_encode_returns(&(true,));
        vm.mock_static_call(contract.verifier.get(), calldata, Ok(return_data));

        let result = contract.cast_vote(proof.into(), proposal_id, U256::from(1), U256::from(5));
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_cast_vote_invalid_proof() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let mut contract = Voting::from(&vm);
        contract.verifier.set(MOCK_VERIFIER);
        let voters_root = U256::from(1);

        // propose a proposal
        let proposal_id =
            contract.propose("Test Proposal".to_string(), U256::from(1000), voters_root);

        // mock proofs and validation call
        let proof = vec![1u8, 2, 3, 4];
        let vote = U256::from(1);
        let nullifier_hash = U256::from(5);

        // build calldata for mock
        let calldata = verifyCall {
            proof: proof.clone().into(),
            input: Vec::from(
                [
                    voters_root.to_be_bytes::<32>(),
                    nullifier_hash.to_be_bytes::<32>(),
                    proposal_id.to_be_bytes::<32>(),
                    vote.to_be_bytes::<32>(),
                ]
                .concat(),
            )
            .into(),
        }
        .abi_encode();
        let return_data = verifyCall::abi_encode_returns(&(false,));
        vm.mock_static_call(contract.verifier.get(), calldata, Ok(return_data));

        let result = contract.cast_vote(proof.into(), proposal_id, vote, nullifier_hash);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap(), b"Invalid proof".to_vec());
    }
}

