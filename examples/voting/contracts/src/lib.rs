// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::{string::String, vec::Vec};
use alloy_primitives::{Address, B256};
use alloy_sol_types::{sol, SolCall, SolType};
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::U256,
    call::static_call,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageString, StorageU256, StorageVec},
};

#[allow(deprecated)]
use stylus_sdk::call::Call as InterfaceCall;

sol! {
    // interface IUltraVerifier {
        function verify(bytes memory proof, bytes memory input) external view returns (bool);
    // }
}

sol! {
    #[derive(Debug, AbiType)]
    struct Proposal {
        string description;
        uint256 deadline;
        uint256 for_votes;
        uint256 against_votes;
    }

    #[derive(Debug)]
    error ProposalNotFound();
    #[derive(Debug)]
    error VotingPeriodOver();
    #[derive(Debug)]
    error ProofAlreadySubmitted();
    #[derive(Debug)]
    error InvalidProof();
}

#[derive(SolidityError, Debug)]
pub enum VotingErrors {
    ProposalNotFound(ProposalNotFound),
    VotingPeriodOver(VotingPeriodOver),
    ProofAlreadySubmitted(ProofAlreadySubmitted),
    InvalidProof(InvalidProof),
}

#[storage]
struct StorageProposal {
    description: StorageString,
    deadline: StorageU256,
    for_votes: StorageU256,
    against_votes: StorageU256,
    started: StorageBool,
}

#[storage]
#[entrypoint]
struct Voting {
    merkle_root: StorageU256,
    verifier: StorageAddress,
    proposal_count: StorageU256,
    proposals: StorageMap<U256, StorageProposal>,
    nullifiers: StorageMap<U256, StorageBool>,
}

#[public]
impl Voting {
    #[constructor]
    pub fn constructor(&mut self, merkle_root: U256, verifier: Address) {
        self.merkle_root.set(merkle_root);
        self.verifier.set(verifier);
        self.proposal_count.set(U256::ZERO);
    }

    pub fn propose(&mut self, description: String, deadline: U256) -> U256 {
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
    ) -> Result<bool, VotingErrors> {
        // Check if the proposal exists and is started
        let proposal = self.proposals.get(proposal_id);
        if !proposal.started.get() {
            return Err(VotingErrors::ProposalNotFound(ProposalNotFound {}));
        }

        // Check if the voting period is over
        if U256::from(self.vm().block_timestamp()) >= proposal.deadline.get() {
            return Err(VotingErrors::VotingPeriodOver(VotingPeriodOver {}));
        }

        // Check if the nullifier hash has already been used
        if self.nullifiers.get(nullifier_hash) {
            return Err(VotingErrors::ProofAlreadySubmitted(
                ProofAlreadySubmitted {},
            ));
        }
        self.nullifiers.insert(nullifier_hash, true);

        // format public inputs
        let mut public_inputs = Vec::new();
        public_inputs.extend_from_slice(&self.merkle_root.get().to_be_bytes::<32>());
        public_inputs.extend_from_slice(&proposal_id.to_be_bytes::<32>());
        public_inputs.extend_from_slice(&vote.to_be_bytes::<32>());
        public_inputs.extend_from_slice(&nullifier_hash.to_be_bytes::<32>());

        // verify the proof. TODO: fix regarding mocks
        // if !IUltraVerifier::new(self.verifier.get())
        //     .verify(
        //         #[allow(deprecated)]
        //         InterfaceCall::new(),
        //         proof.to_vec().into(),
        //         public_inputs.into(),
        //     )
        //     .unwrap()
        // {
        //     return Err(VotingErrors::InvalidProof(InvalidProof {}));
        // }
        // IUltraVerifier::new(self.verifier.get()).verify(self.vm(), proof.to_vec().into(), public_inputs.into());
        // static_call_helper::<verify>(self, self.verifier.get(), (proof.into(), public_inputs.into()))
        // .map(|res| res._0)?;
        // static_call(&*self, self.verifier.get(), &vec![1,2,3,4]).unwrap();

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
}

// #[allow(deprecated)]
// pub fn static_call_helper<C: SolCall>(
//     storage: &impl TopLevelStorage,
//     address: Address,
//     args: <C::Parameters<'_> as SolType>::RustType,
// ) -> Result<C::Return, Vec<u8>> {
//     let calldata = C::new(args).abi_encode();
//     let res = static_call(storage, address, &calldata)?;
//     C::abi_decode_returns(&res, false /* validate */).map_err(|_| b"Demo".to_vec())
// }

#[cfg(test)]
mod test {
    use super::*;
    use alloy_primitives::address;

    #[test]
    fn test_propose_returns_proposal_id() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let mut contract = Voting::from(&vm);

        let proposal_id = contract.propose("Test Proposal".to_string(), U256::from(1000));
        assert_eq!(proposal_id, U256::from(0));
    }

    #[test]
    fn test_propose_stores_proposal() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let mut contract = Voting::from(&vm);

        let proposal_id = contract.propose("Test Proposal".to_string(), U256::from(1000));
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
        contract
            .verifier
            .set(address!("0x0000000000000000000000000000000000000001"));

        // propose a proposal
        let proposal_id = contract.propose("Test Proposal".to_string(), U256::from(1000));

        // mock proofs and validation call
        let proof = Bytes::from(vec![1u8, 2, 3, 4]);
        let vote = U256::from(1);
        let nullifier_hash = U256::from(5);

        // format public inputs the same way as in cast_vote
        let mut calldata = Vec::new();
        calldata.extend_from_slice(&[0xf7, 0xe8, 0x3a, 0xee]);
        calldata.extend_from_slice(&proof.to_vec());
        calldata.extend_from_slice(&contract.merkle_root.get().to_be_bytes::<32>());
        calldata.extend_from_slice(&proposal_id.to_be_bytes::<32>());
        calldata.extend_from_slice(&vote.to_be_bytes::<32>());
        calldata.extend_from_slice(&nullifier_hash.to_be_bytes::<32>());

        vm.mock_static_call(
            // contract.verifier.get(),
            address!("0x0000000000000000000000000000000000000001"),
            vec![1,2,3,4], // empty calldata for mock
            Ok(vec![1]),
        );

        let result = contract.cast_vote(proof, proposal_id, U256::from(1), U256::from(5));
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), true);
    }
}
