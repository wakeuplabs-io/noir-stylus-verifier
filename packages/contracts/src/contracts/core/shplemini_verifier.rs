use crate::utils::backends::{PrecompileG1ArithmeticBackend, PrecompileHashBackend};
use alloc::vec::Vec;
use stylus_sdk::{abi::Bytes, prelude::*};
use ultrahonk::{constants::get_crs_g2, decider::types::VerifierMemory};
use ultrahonk::decider::verifier::DeciderVerifier;
use ultrahonk::serialize::{BytesDeserializable, BytesSerializable};
use ultrahonk::transcript::Transcript;
use ultrahonk::types::ScalarField;
use ultrahonk::CONST_PROOF_SIZE_LOG_N;
use ark_bn254::{G1Affine, G2Affine};
use ark_ff::{Zero, One};
use ark_ec::AffineRepr;
use ultrahonk::backends::G1ArithmeticBackend;
use crate::alloc::string::ToString;

#[cfg_attr(feature = "shplemini-verifier", entrypoint)]
#[storage]
pub struct ShpleminiVerifierContract {}

#[public]
impl ShpleminiVerifierContract {
    // pub fn verify(
    //     &self,
    //     memory_bytes: Bytes,
    //     transcript_bytes: Bytes,
    //     multivariate_challenge: Bytes,
    //     circuit_size: u32,
    // ) -> bool {
    //     // let memory = VerifierMemory::deserialize_from_bytes(memory_bytes.as_slice()).unwrap();
    //     // let mut transcript =
    //     //     Transcript::deserialize_from_bytes(transcript_bytes.as_slice()).unwrap();
    //     // let multivariate_challenge =
    //     //     Vec::<ScalarField>::deserialize_from_bytes(multivariate_challenge.as_slice()).unwrap();

    //     // let mut decider_verifier =
    //     //     DeciderVerifier::<PrecompileG1ArithmeticBackend, PrecompileHashBackend>::new(memory);

    //     // let shplemini_output = decider_verifier
    //     //     .verify_shplemini(&mut transcript, multivariate_challenge, circuit_size)
    //     //     .unwrap();

    //     let log_circuit_size = circuit_size.ilog2() as usize;

    //     let mut padding_indicator_array = [ScalarField::zero(); CONST_PROOF_SIZE_LOG_N];

    //     for (idx, value) in padding_indicator_array.iter_mut().enumerate() {
    //         *value = if idx < log_circuit_size as usize {
    //             ScalarField::one()
    //         } else {
    //             ScalarField::zero()
    //         };
    //     }

    //     // TODO: call compute_batch_opening_claim

    //     // TODO: call compute_pairing_points
    //     let pairing_points = (G1Affine::zero(), G1Affine::zero());

    //     let pcs_verified = PrecompileG1ArithmeticBackend::ec_pairing_check(
    //         pairing_points.0,
    //         pairing_points.1,
    //         get_crs_g2(),
    //         G2Affine::generator(),
    //     )
    //     .unwrap();


    //     pcs_verified
    // }


    // // compute pairing points
    // pub fn verify(
    //     &self,
    //     memory_bytes: Bytes,
    //     transcript_bytes: Bytes,
    //     multivariate_challenge: Bytes,
    //     circuit_size: u32,
    // ) -> (Bytes, Bytes) {
    //     let memory = VerifierMemory::deserialize_from_bytes(memory_bytes.as_slice()).unwrap();
    //     let mut transcript =
    //         Transcript::deserialize_from_bytes(transcript_bytes.as_slice()).unwrap();
    //     let multivariate_challenge =
    //         Vec::<ScalarField>::deserialize_from_bytes(multivariate_challenge.as_slice()).unwrap();

    //     let decider_verifier =
    //         DeciderVerifier::<PrecompileG1ArithmeticBackend, PrecompileHashBackend>::new(memory);

    //        let mut opening_pair = decider_verifier.compute_batch_opening_claim(
    //         multivariate_challenge,
    //         &mut transcript,
    //         &[ScalarField::zero(); CONST_PROOF_SIZE_LOG_N],
    //     ).unwrap();

    //     let quotient_commitment = transcript.receive_point_from_prover("KZG:W".to_string()).unwrap();
    //     opening_pair.commitments.push(quotient_commitment);
    //     opening_pair.scalars.push(opening_pair.challenge);

    //     let p_1 = -quotient_commitment.into_group();
    //     let p_0 = PrecompileG1ArithmeticBackend::msm(&opening_pair.scalars, &opening_pair.commitments)
    //         .unwrap();

    //     let p_0_affine: G1Affine = p_0.into();
    //     let p_1_affine: G1Affine = p_1.into();

    //     (p_0_affine.serialize_to_bytes().into(), p_1_affine.serialize_to_bytes().into())
    // }



    // compute opening claim
    pub fn verify(
        &self,
        memory_bytes: Bytes,
        transcript_bytes: Bytes,
        multivariate_challenge: Bytes,
        circuit_size: u32,
    ) -> bool {
        let memory = VerifierMemory::deserialize_from_bytes(memory_bytes.as_slice()).unwrap();
        let mut transcript =
            Transcript::deserialize_from_bytes(transcript_bytes.as_slice()).unwrap();
        let multivariate_challenge =
            Vec::<ScalarField>::deserialize_from_bytes(multivariate_challenge.as_slice()).unwrap();

        let mut decider_verifier = DeciderVerifier::new(memory);
        decider_verifier.verify_shplemini::<PrecompileHashBackend, PrecompileG1ArithmeticBackend>(
            &mut transcript,
            multivariate_challenge,
            circuit_size,
        ).unwrap();

        true
    }
}
