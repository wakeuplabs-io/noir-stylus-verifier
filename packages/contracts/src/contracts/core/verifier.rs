use crate::utils::backends::{PrecompileG1ArithmeticBackend, PrecompileHashBackend};
use alloc::vec::Vec;
use alloy_primitives::Address;
use stylus_sdk::call::{Call, RawCall};
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::decider::sumcheck::round_verifier::{InlineSumcheckVerifier, SumcheckVerifierRound};
use ultrahonk::decider::types::{ClaimedEvaluations, RelationParameters, VerifierMemory};
use ultrahonk::decider::verifier::DeciderVerifier;
use ultrahonk::keys::verification_key::VerifyingKey;
use ultrahonk::serialize::{BytesDeserializable, BytesSerializable};
use ultrahonk::transcript::Transcript;
use ultrahonk::types::{HonkProof, ScalarField};

sol_storage! {
    #[cfg_attr(feature = "verifier", entrypoint)]
    pub struct VerifierContract {
        bool initialized;
        address sumcheck_verifier_address;
        address shplemini_verifier_address;
    }
}

sol_interface! {
    interface ISumcheckVerifier {
        function verify(bytes memory mem, bytes memory transcript, uint32 circuit_size) external returns (bytes memory, bytes memory, bool);
    }
    interface IShpleminiVerifier {
        function verify(bytes memory mem, bytes memory transcript, bytes memory multivariate_challenge, uint32 circuit_size) external returns (bool);
    }
}

#[public]
impl VerifierContract {
    /// The constructor sets the owner as the EOA that deployed the contract.
    pub fn initialize(
        &mut self,
        sumcheck_verifier_address: Address,
        shplemini_verifier_address: Address,
    ) {
        if self.initialized.get() {
            return;
        }

        self.sumcheck_verifier_address
            .set(sumcheck_verifier_address);
        self.shplemini_verifier_address
            .set(shplemini_verifier_address);

        // mark as initialized
        self.initialized.set(true);
    }

    pub fn verify(
        &mut self,
        proof_bytes: Bytes,
        public_inputs_bytes: Bytes,
        vk_bytes: Bytes,
    ) -> bool {
        // parse public_inputs file
        let public_inputs =
            Vec::<ScalarField>::deserialize_from_bytes(&public_inputs_bytes).unwrap();

        // parse proof file
        let proof = HonkProof::from_buffer(&proof_bytes).unwrap();
        let proof = proof.insert_public_inputs(public_inputs);

        // parse verification key file
        let vk = VerifyingKey::from_buffer(&vk_bytes).unwrap();

        // create transcript and memory
        let mut transcript = Transcript::new_verifier(proof);
        let memory =
            VerifierMemory::from_key_and_transcript::<PrecompileHashBackend>(&vk, &mut transcript);
        let memory_bytes = memory.serialize_to_bytes();

        

        // // sumcheck verification
        let sumcheck_verifier = ISumcheckVerifier::new(self.sumcheck_verifier_address.get());
        let (_, _, sumcheck_ok) = sumcheck_verifier
            .verify(
                Call::new(),
                memory_bytes.to_vec().into(),
                transcript.serialize_to_bytes().into(),
                vk.circuit_size.into(),
            )
            .unwrap();

        // if !sumcheck_ok {
        //     return false;
        // }

        // // shplemini verification

        // let shplemini_verifier = IShpleminiVerifier::new(self.shplemini_verifier_address.get());

        // let (shplemini_ok) = shplemini_verifier
        //     .verify(
        //         Call::new(),
        //         memory_bytes.to_vec().into(),
        //         transcript_bytes.to_vec().into(),
        //         multivariate_challenge.to_vec().into(),
        //         vk.circuit_size.into(),
        //     )
        //     .unwrap();


        // let mut decider_verifier = DeciderVerifier::new(memory);
        // let sumcheck_output = decider_verifier.verify_sumcheck::<PrecompileHashBackend, InlineSumcheckVerifier>(&mut transcript, vk.circuit_size).unwrap();
        // let shplemini_output = decider_verifier.verify_shplemini::<PrecompileHashBackend, PrecompileG1ArithmeticBackend>(
        //     &mut transcript,
        //     sumcheck_output.multivariate_challenge,
        //     vk.circuit_size,
        // ).unwrap();

        true
    }

    pub fn get_sumcheck_verifier_address(&self) -> Address {
        self.sumcheck_verifier_address.get()
    }

    pub fn get_shplemini_verifier_address(&self) -> Address {
        self.shplemini_verifier_address.get()
    }
}
