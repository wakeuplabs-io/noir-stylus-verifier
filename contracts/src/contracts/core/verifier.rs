//! Verifier contract, entry point for the verifier. Calls the sumcheck and shplemini verifiers.

use crate::must_deser;
use crate::utils::backends::PrecompileHashBackend;
use alloc::vec::Vec;
use alloy_primitives::Address;
use alloy_sol_types::sol;
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::keys::verification_key::VerifyingKey;
use ultrahonk::oink::verifier::OinkVerifier;
use ultrahonk::serialize::BytesSerializable;
use ultrahonk::transcript::Transcript;
use ultrahonk::types::{HonkProof, ScalarField};
use ultrahonk::CONST_PROOF_SIZE_LOG_N;

#[cfg(any(feature = "zk-verifier"))]
use ultrahonk::decider::shplemini::verifier::ShpleminiVerifier;

#[allow(deprecated)]
use stylus_sdk::call::Call as InterfaceCall;

sol_storage! {
    #[cfg_attr(any(feature = "verifier", feature = "zk-verifier"), entrypoint)]
    pub struct VerifierContract {
        address sumcheck_verifier_address;
        address shplemini_verifier_address;
    }
}

// Define errors that can occur during the execution of the contract
sol! {
    error SumcheckFailed();
    error ShpleminiFailed();
    error ConsistencyFailed();
}

#[derive(SolidityError)]
pub enum VerifierErrors {
    SumcheckFailed(SumcheckFailed),
    ShpleminiFailed(ShpleminiFailed),
    ConsistencyFailed(ConsistencyFailed),
}

// Define interfaces for the verifiers
sol_interface! {
    interface ISumcheckVerifier {
        function verify(
            bytes memory challenges,
            bytes memory public_input_delta,
            bytes memory gate_challenges,
            bytes memory transcript,
            uint32 circuit_size
        ) external returns (bool, bytes memory, bytes memory, bytes memory, bytes memory, bytes memory);
    }
    interface IShpleminiVerifier {
        function verify(
            bytes memory transcript_bytes,
            bytes memory witness_commitments_bytes,
            bytes memory verifier_commitments_bytes,
            bytes memory claimed_evaluations_bytes,
            bytes memory multivariate_challenge_bytes,
            bytes memory libra_commitments_bytes,
            uint32 circuit_size
        ) external returns (bool, bytes memory, bytes memory);
    }
}

#[public]
impl VerifierContract {
    #[constructor]
    pub fn constructor(
        &mut self,
        sumcheck_verifier_address: Address,
        shplemini_verifier_address: Address,
    ) {
        self.sumcheck_verifier_address
            .set(sumcheck_verifier_address);
        self.shplemini_verifier_address
            .set(shplemini_verifier_address);
    }

    pub fn verify(
        &mut self,
        proof_bytes: Bytes,
        public_inputs_bytes: Bytes,
        vk_bytes: Bytes,
    ) -> Result<bool, VerifierErrors> {
        // deserialize parameters
        let public_inputs = must_deser!(Vec<ScalarField>, public_inputs_bytes);
        let vk = must_deser!(VerifyingKey, vk_bytes);
        let proof = must_deser!(HonkProof, proof_bytes);

        // create transcript and memory
        let mut transcript = Transcript::new_verifier(proof.insert_public_inputs(public_inputs));

        let oink_verifier = OinkVerifier::default();
        let verifier_memory = oink_verifier
            .verify::<PrecompileHashBackend>(&vk, &mut transcript)
            .unwrap();

        let mut gate_challenges: Vec<ScalarField> = Vec::with_capacity(CONST_PROOF_SIZE_LOG_N);
        for _ in 0..CONST_PROOF_SIZE_LOG_N {
            let chall = transcript.get_challenge::<PrecompileHashBackend>(); // format!("Sumcheck:gate_challenge_{}", idx)
            gate_challenges.push(chall);
        }

        // sumcheck verification
        let (
            sumcheck_ok,
            claimed_evaluations_bytes,
            transcript_bytes,
            multivariate_challenge,
            claimed_libra_evaluation,
            libra_commitments,
        ) = ISumcheckVerifier::new(self.sumcheck_verifier_address.get())
            .verify(
                #[allow(deprecated)]
                InterfaceCall::new(),
                transcript.serialize_to_bytes().into(),
                verifier_memory.challenges.serialize_to_bytes().into(),
                verifier_memory
                    .public_input_delta
                    .serialize_to_bytes()
                    .into(),
                gate_challenges.serialize_to_bytes().into(),
                vk.circuit_size,
            )
            .map_err(|_| VerifierErrors::SumcheckFailed(SumcheckFailed {}))?;
        if !sumcheck_ok {
            return Err(VerifierErrors::SumcheckFailed(SumcheckFailed {}));
        }

        // shplemini verification
        let (shplemini_ok, libra_evaluations, gemini_evaluation_challenge) =
            IShpleminiVerifier::new(self.shplemini_verifier_address.get())
                .verify(
                    #[allow(deprecated)]
                    InterfaceCall::new(),
                    transcript_bytes.to_vec().into(),
                    verifier_memory
                        .witness_commitments
                        .serialize_to_bytes()
                        .into(),
                    vk.commitments.serialize_to_bytes().into(),
                    claimed_evaluations_bytes.to_vec().into(),
                    multivariate_challenge.to_vec().into(),
                    libra_commitments.to_vec().into(),
                    vk.circuit_size,
                )
                .map_err(|_| VerifierErrors::ShpleminiFailed(ShpleminiFailed {}))?;
        if !shplemini_ok {
            return Err(VerifierErrors::ShpleminiFailed(ShpleminiFailed {}));
        }

        #[cfg(any(feature = "zk-verifier"))]
        {
            let consistency_checked = ShpleminiVerifier::check_evaluations_consistency(
                &must_deser!(Vec<ScalarField>, libra_evaluations),
                must_deser!(ScalarField, gemini_evaluation_challenge),
                &must_deser!(Vec<ScalarField>, multivariate_challenge),
                must_deser!(ScalarField, claimed_libra_evaluation),
            )
            .map_err(|_| VerifierErrors::ConsistencyFailed(ConsistencyFailed {}))?;

            if !consistency_checked {
                return Err(VerifierErrors::ConsistencyFailed(ConsistencyFailed {}));
            }
        }

        Ok(true) // reverts if any of the verifications failed
    }

    pub fn get_sumcheck_verifier_address(&self) -> Address {
        self.sumcheck_verifier_address.get()
    }

    pub fn get_shplemini_verifier_address(&self) -> Address {
        self.shplemini_verifier_address.get()
    }
}
