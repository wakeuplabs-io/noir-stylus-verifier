// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::{string::String, vec::Vec};
use alloy_primitives::Address;
use alloy_sol_types::sol;
use alloy_sol_types::SolCall;
use alloy_sol_types::SolType;
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::U256,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageString, StorageU256},
    stylus_core::calls::context::Call,
};

sol! {
    #[derive(Debug)]
    struct Proposal {
        string metadata; // cid to ipfs metadata
        uint256 voters_root;
        uint256 for_votes;
        uint256 against_votes;
        address author;
        uint256 deadline;
        uint256 created_at;
    }

    event ProposalCreated(uint256 indexed id);
    event NullifierUsed(uint256 indexed nullifier_hash);

    function verify(bytes memory proof, bytes memory input) external view returns (bool);
}

sol_interface! {
    interface IUltraVerifier {
        function verify(bytes memory proof, bytes memory input) external view returns (bool);
    }
}

#[storage]
struct StorageProposal {
    metadata: StorageString,
    for_votes: StorageU256,
    against_votes: StorageU256,
    started: StorageBool,
    voters_root: StorageU256,
    author: StorageAddress,
    created_at: StorageU256,
    deadline: StorageU256,
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

    /// Get the verifier address
    /// @return The verifier address
    pub fn get_verifier(&self) -> Address {
        self.verifier.get()
    }

    /// Get the number of proposals
    /// @return The number of proposals
    pub fn get_proposal_count(&self) -> U256 {
        self.proposal_count.get()
    }

    /// Weather or not a nullifier has been used
    /// @param nullifier_hash - The hash of the nullifier
    /// @return True if the nullifier has been used, false otherwise
    pub fn is_nullifier_used(&self, nullifier_hash: U256) -> bool {
        self.nullifiers.get(nullifier_hash)
    }

    /// Propose a new proposal
    /// @param metadata - The ipfs cid containing the metadata of the proposal
    /// @param deadline - The deadline of the proposal
    /// @param voters_root - The merkle root containing the voters
    pub fn propose(&mut self, metadata: String, deadline: U256, voters_root: U256) {
        let proposal_id = self.proposal_count.get();
        let created_at = U256::from(self.vm().block_timestamp().clone());
        let author = self.vm().msg_sender();

        // store the proposal in the storage
        self.proposals
            .setter(proposal_id)
            .metadata
            .set_str(&metadata);
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
        self.proposals.setter(proposal_id).deadline.set(deadline);
        self.proposals.setter(proposal_id).author.set(author);
        self.proposals
            .setter(proposal_id)
            .created_at
            .set(created_at);

        // increment the proposal count and return the id
        self.proposal_count.set(proposal_id + U256::from(1));

        // log the proposal created event so frontend can retrieve id
        log(self.vm(), ProposalCreated { id: proposal_id });
    }

    /// FIXME: returning Proposal as a struct seems to be not quite working as of stylus 0.9.0. Revising when updating to 0.10.0.

    /// Get the metadata of a proposal
    /// @param proposal_id - The id of the proposal
    /// @return The metadata of the proposal
    pub fn get_proposal_metadata(&self, proposal_id: U256) -> String {
        self.proposals.get(proposal_id).metadata.get_string()
    }

    /// Get the deadline of a proposal
    /// @param proposal_id - The id of the proposal
    /// @return The deadline of the proposal
    pub fn get_proposal_deadline(&self, proposal_id: U256) -> U256 {
        self.proposals.get(proposal_id).deadline.get()
    }

    /// Get the number of for votes for a proposal
    /// @param proposal_id - The id of the proposal
    /// @return The number of for votes for the proposal
    pub fn get_proposal_for_votes(&self, proposal_id: U256) -> U256 {
        self.proposals.get(proposal_id).for_votes.get()
    }

    /// Get the number of against votes for a proposal
    /// @param proposal_id - The id of the proposal
    /// @return The number of against votes for the proposal
    pub fn get_proposal_against_votes(&self, proposal_id: U256) -> U256 {
        self.proposals.get(proposal_id).against_votes.get()
    }

    /// Get the voters root for a proposal
    /// @param proposal_id - The id of the proposal
    /// @return The voters root for the proposal
    pub fn get_proposal_voters_root(&self, proposal_id: U256) -> U256 {
        self.proposals.get(proposal_id).voters_root.get()
    }

    /// Get the author of a proposal
    /// @param proposal_id - The id of the proposal
    /// @return The author of the proposal
    pub fn get_proposal_author(&self, proposal_id: U256) -> Address {
        self.proposals.get(proposal_id).author.get()
    }

    /// Get the created at timestamp of a proposal
    /// @param proposal_id - The id of the proposal
    /// @return The created at timestamp of the proposal
    pub fn get_proposal_created_at(&self, proposal_id: U256) -> U256 {
        self.proposals.get(proposal_id).created_at.get()
    }

    /// Cast a vote for a proposal
    /// @param proof - The proof of the vote
    /// @param proposal_id - The id of the proposal
    /// @param vote - The vote (1 for for, 0 for against)
    /// @param nullifier_hash - The hash of the nullifier
    /// @return True if the vote was cast successfully, false otherwise
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
        if !static_call_helper::<verifyCall>(
            self.vm(),
            self.verifier.get(),
            (
                proof.to_vec().into(),
                Vec::from(
                    [
                        proposal.voters_root.get().to_be_bytes::<32>(),
                        nullifier_hash.to_be_bytes::<32>(),
                        proposal_id.to_be_bytes::<32>(),
                        vote.to_be_bytes::<32>(),
                    ]
                    .concat(),
                )
                .into(),
            ),
        )
        .map(|res| res._0)?
        {
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

        log(self.vm(), NullifierUsed { nullifier_hash });

        Ok(true)
    }
}

pub fn static_call_helper<C: SolCall>(
    vm: &dyn Host,
    address: Address,
    args: <C::Parameters<'_> as SolType>::RustType,
) -> Result<C::Return, Vec<u8>> {
    let calldata = C::new(args).abi_encode();
    let res = vm
        .static_call(&Call::new(), address, &calldata)
        .map_err(|_| b"Call failed".to_vec())?;
    C::abi_decode_returns(&res, false).map_err(|_| b"Failed to decode return data".to_vec())
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
    fn test_propose() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let mut contract = Voting::from(&vm);

        // create proposal
        contract.propose("Test Proposal".to_string(), U256::from(1000), U256::from(1));

        // check we emitted event
        let events = vm.get_emitted_logs();
        assert_eq!(events.len(), 1);

        // check we incremented the proposal count
        assert_eq!(contract.proposal_count.get(), U256::from(1));
    }

    #[test]
    fn test_propose_stores_proposal() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let mut contract = Voting::from(&vm);

        contract.propose("cid".to_string(), U256::from(1000), U256::from(1));

        assert_eq!(contract.get_proposal_metadata(U256::from(0)), "cid");
        assert_eq!(
            contract.get_proposal_deadline(U256::from(0)),
            U256::from(1000)
        );
        assert_eq!(contract.get_proposal_for_votes(U256::from(0)), U256::ZERO);
        assert_eq!(
            contract.get_proposal_against_votes(U256::from(0)),
            U256::ZERO
        );
        assert_eq!(
            contract.get_proposal_voters_root(U256::from(0)),
            U256::from(1)
        );
    }

    #[test]
    fn test_cast_vote_returns_true() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let mut contract = Voting::from(&vm);
        contract.verifier.set(MOCK_VERIFIER);
        let voters_root = U256::from(1);

        // propose a proposal
        contract.propose("cid".to_string(), U256::from(1000), voters_root);

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
                    U256::from(0).to_be_bytes::<32>(),
                    vote.to_be_bytes::<32>(),
                ]
                .concat(),
            )
            .into(),
        }
        .abi_encode();
        let return_data = verifyCall::abi_encode_returns(&(true,));
        vm.mock_static_call(contract.verifier.get(), calldata, Ok(return_data));

        let result = contract.cast_vote(proof.into(), U256::from(0), U256::from(1), U256::from(5));
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
        contract.propose("cid".to_string(), U256::from(1000), voters_root);

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
                    U256::from(0).to_be_bytes::<32>(),
                    vote.to_be_bytes::<32>(),
                ]
                .concat(),
            )
            .into(),
        }
        .abi_encode();
        let return_data = verifyCall::abi_encode_returns(&(false,));
        vm.mock_static_call(contract.verifier.get(), calldata, Ok(return_data));

        let result = contract.cast_vote(proof.into(), U256::from(0), vote, nullifier_hash);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap(), b"Invalid proof".to_vec());
    }
}
