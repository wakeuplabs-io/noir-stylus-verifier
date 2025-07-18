use crate::{
    backends::{G1ArithmeticBackend, HashBackend},
    constants::{BATCHED_RELATION_PARTIAL_LENGTH, BATCHED_RELATION_PARTIAL_LENGTH_ZK},
    decider::{
        shplemini::verifier::{ShpleminiVerifier, ShpleminiVerifierMemory},
        sumcheck::verifier::{SumcheckVerifier, SumcheckVerifierMemory},
    },
    oink::verifier::OinkVerifier,
    transcript::Transcript,
    types::{HonkProof, HonkVerifyResult, ScalarField, VerifyingKey},
    CONST_PROOF_SIZE_LOG_N,
};

pub struct UltraHonk;

impl UltraHonk {
    pub fn verify<H: HashBackend, P: G1ArithmeticBackend>(
        honk_proof: HonkProof,
        public_inputs: &[ScalarField],
        vk: &VerifyingKey,
        has_zk: bool,
    ) -> HonkVerifyResult<bool> {
        let mut transcript =
            Transcript::new_verifier(honk_proof.insert_public_inputs(public_inputs.to_vec()));

        let oink_verifier = OinkVerifier::default();
        let oink_memory = oink_verifier.verify::<H>(vk, &mut transcript).unwrap();

        let mut gate_challenges: Vec<ScalarField> = Vec::with_capacity(CONST_PROOF_SIZE_LOG_N);
        for _ in 0..CONST_PROOF_SIZE_LOG_N {
            let chall = transcript.get_challenge::<H>(); // format!("Sumcheck:gate_challenge_{}", idx)
            gate_challenges.push(chall);
        }

        // build sumcheck verifier
        let mut sumcheck_verifier = SumcheckVerifier::new(
            SumcheckVerifierMemory::from_memory_and_gate_challenges::<H>(
                &oink_memory.challenges,
                gate_challenges,
                oink_memory.public_input_delta,
            ),
        );
        let (sumcheck_output, libra_commitments) = if has_zk {
            sumcheck_verifier.verify_sumcheck::<H, BATCHED_RELATION_PARTIAL_LENGTH_ZK>(
                &mut transcript,
                vk.circuit_size,
                has_zk,
            )?
        } else {
            sumcheck_verifier.verify_sumcheck::<H, BATCHED_RELATION_PARTIAL_LENGTH>(
                &mut transcript,
                vk.circuit_size,
                has_zk,
            )?
        };
        if !sumcheck_output.verified {
            return Ok(false);
        }

        // build shplemini verifier
        let mut shplemini_verifier = ShpleminiVerifier::new(ShpleminiVerifierMemory::new(
            oink_memory.witness_commitments.clone(),
            vk.commitments.clone(),
            sumcheck_verifier.memory.claimed_evaluations.clone(),
        ));
        let (pcs_verified, libra_evaluations, gemini_evaluation_challenge) = shplemini_verifier
            .verify_shplemini::<H, P>(
                &mut transcript,
                sumcheck_output.multivariate_challenge.clone(),
                vk.circuit_size,
                libra_commitments,
            )?;
        if !pcs_verified {
            return Ok(false);
        }

        // consistency check
        let mut consistency_checked = true;
        if has_zk {
            consistency_checked = ShpleminiVerifier::check_evaluations_consistency(
                &libra_evaluations,
                gemini_evaluation_challenge,
                &sumcheck_output.multivariate_challenge,
                sumcheck_output.claimed_libra_evaluation,
            )?;
        }

        Ok(sumcheck_output.verified && pcs_verified && consistency_checked)
    }
}
