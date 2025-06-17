use super::types::VerifierMemory;
use crate::{
    backends::HashBackend, honk_curve::HonkCurve, keys::verification_key::VerifyingKey,
    transcript::Transcript, types::ScalarField, verifier::HonkVerifyResult, NUM_ALPHAS,
};
use alloc::vec::Vec;
use ark_ff::One;
use core::{array, marker::PhantomData};

pub(crate) struct Oink<P: HonkCurve, H: HashBackend> {
    phantom_data: PhantomData<P>,
    phantom_hasher: PhantomData<H>,
}

impl<P: HonkCurve, H: HashBackend> Default for Oink<P, H> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: HonkCurve, H: HashBackend> Oink<P, H> {
    pub(crate) fn new() -> Self {
        Self {
            phantom_data: PhantomData,
            phantom_hasher: PhantomData,
        }
    }

    pub(crate) fn compute_public_input_delta(
        beta: &ScalarField,
        gamma: &ScalarField,
        public_inputs: &[ScalarField],
        circuit_size: u32,
        pub_inputs_offset: u32,
    ) -> ScalarField {
        tracing::trace!("compute public input delta");

        // Let m be the number of public inputs x₀,…, xₘ₋₁.
        // Recall that we broke the permutation σ⁰ by changing the mapping
        //  (i) -> (n+i)   to   (i) -> (-(i+1))   i.e. σ⁰ᵢ = −(i+1)
        //
        // Therefore, the term in the numerator with ID¹ᵢ = n+i does not cancel out with any term in the denominator.
        // Similarly, the denominator contains an extra σ⁰ᵢ = −(i+1) term that does not appear in the numerator.
        // We expect the values of W⁰ᵢ and W¹ᵢ to be equal to xᵢ.
        // The expected accumulated product would therefore be equal to

        //   ∏ᵢ (γ + W¹ᵢ + β⋅ID¹ᵢ)        ∏ᵢ (γ + xᵢ + β⋅(n+i) )
        //  -----------------------  =  ------------------------
        //   ∏ᵢ (γ + W⁰ᵢ + β⋅σ⁰ᵢ )        ∏ᵢ (γ + xᵢ - β⋅(i+1) )

        // At the start of the loop for each xᵢ where i = 0, 1, …, m-1,
        // we have
        //      numerator_acc   = γ + β⋅(n+i) = γ + β⋅n + β⋅i
        //      denominator_acc = γ - β⋅(1+i) = γ - β   - β⋅i
        // at the end of the loop, add and subtract β to each term respectively to
        // set the expected value for the start of iteration i+1.
        // Note: The public inputs may be offset from the 0th index of the wires, for example due to the inclusion of an
        // initial zero row or Goblin-stlye ECC op gates. Accordingly, the indices i in the above formulas are given by i =
        // [0, m-1] + offset, i.e. i = offset, 1 + offset, …, m - 1 + offset.

        let mut num = ScalarField::one();
        let mut denom = ScalarField::one();
        let mut num_acc =
            *gamma + ScalarField::from((circuit_size + pub_inputs_offset) as u64) * beta;
        let mut denom_acc = *gamma - ScalarField::from((1 + pub_inputs_offset) as u64) * beta;

        for x_i in public_inputs.iter() {
            num *= num_acc + x_i;
            denom *= denom_acc + x_i;
            num_acc += beta;
            denom_acc -= beta;
        }
        num / denom
    }

    /// Generate relation separators alphas for sumcheck/combiner computation
    pub(crate) fn generate_alphas_round(
        alphas: &mut [ScalarField; NUM_ALPHAS],
        transcript: &mut Transcript<H>,
    ) {
        tracing::trace!("generate alpha round");

        let args: [String; NUM_ALPHAS] = array::from_fn(|i| format!("alpha_{}", i));
        alphas.copy_from_slice(&transcript.get_challenges(&args));
    }
}

pub(crate) struct OinkVerifier<P: HonkCurve, H: HashBackend> {
    memory: VerifierMemory,
    pub public_inputs: Vec<ScalarField>,
    phantom_hasher: core::marker::PhantomData<H>,
    phantom_curve: core::marker::PhantomData<P>,
}

impl<P: HonkCurve, H: HashBackend> Default for OinkVerifier<P, H> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: HonkCurve, H: HashBackend> OinkVerifier<P, H> {
    pub(crate) fn new() -> Self {
        Self {
            memory: VerifierMemory::default(),
            public_inputs: Default::default(),
            phantom_hasher: Default::default(),
            phantom_curve: Default::default(),
        }
    }

    fn execute_preamble_round(
        &mut self,
        verifying_key: &VerifyingKey,
        transcript: &mut Transcript<H>,
    ) -> HonkVerifyResult<()> {
        tracing::trace!("executing (verifying) preamble round");

        let circuit_size = verifying_key.circuit_size as u64;
        let public_input_size = verifying_key.num_public_inputs as u64;
        let pub_inputs_offset = verifying_key.pub_inputs_offset as u64;

        transcript.add_u64_to_hash_buffer("circuit_size".to_string(), circuit_size);
        transcript.add_u64_to_hash_buffer("public_input_size".to_string(), public_input_size);
        transcript.add_u64_to_hash_buffer("pub_inputs_offset".to_string(), pub_inputs_offset);

        self.public_inputs = Vec::with_capacity(public_input_size as usize);

        for i in 0..public_input_size {
            let public_input = transcript.receive_fr_from_prover(format!("public_input_{}", i))?;
            self.public_inputs.push(public_input);
        }

        Ok(())
    }

    fn execute_wire_commitments_round(
        &mut self,
        transcript: &mut Transcript<H>,
    ) -> HonkVerifyResult<()> {
        tracing::trace!("executing (verifying) wire commitments round");

        *self.memory.witness_commitments.w_l_mut() =
            transcript.receive_point_from_prover("W_L".to_string())?;
        *self.memory.witness_commitments.w_r_mut() =
            transcript.receive_point_from_prover("W_R".to_string())?;
        *self.memory.witness_commitments.w_o_mut() =
            transcript.receive_point_from_prover("W_O".to_string())?;

        // Round is done since ultra_honk is no goblin flavor
        Ok(())
    }

    fn execute_sorted_list_accumulator_round(
        &mut self,
        transcript: &mut Transcript<H>,
    ) -> HonkVerifyResult<()> {
        tracing::trace!("executing (verifying) sorted list accumulator round");

        let challs = transcript.get_challenges(&[
            "eta".to_string(),
            "eta_two".to_string(),
            "eta_three".to_string(),
        ]);
        self.memory.challenges.eta_1 = challs[0];
        self.memory.challenges.eta_2 = challs[1];
        self.memory.challenges.eta_3 = challs[2];

        *self.memory.witness_commitments.lookup_read_counts_mut() =
            transcript.receive_point_from_prover("lookup_read_counts".to_string())?;

        *self.memory.witness_commitments.lookup_read_tags_mut() =
            transcript.receive_point_from_prover("lookup_read_tags".to_string())?;

        *self.memory.witness_commitments.w_4_mut() =
            transcript.receive_point_from_prover("w_4".to_string())?;

        Ok(())
    }

    fn execute_log_derivative_inverse_round(
        &mut self,
        transcript: &mut Transcript<H>,
    ) -> HonkVerifyResult<()> {
        tracing::trace!("executing (verifying) log derivative inverse round");

        let challs = transcript.get_challenges(&["beta".to_string(), "gamma".to_string()]);
        self.memory.challenges.beta = challs[0];
        self.memory.challenges.gamma = challs[1];

        *self.memory.witness_commitments.lookup_inverses_mut() =
            transcript.receive_point_from_prover("lookup_inverses".to_string())?;

        // Round is done since ultra_honk is no goblin flavor
        Ok(())
    }

    fn execute_grand_product_computation_round(
        &mut self,
        verifying_key: &VerifyingKey,
        transcript: &mut Transcript<H>,
    ) -> HonkVerifyResult<()> {
        tracing::trace!("executing (verifying) grand product computation round");
        self.memory.public_input_delta = Oink::<P, H>::compute_public_input_delta(
            &self.memory.challenges.beta,
            &self.memory.challenges.gamma,
            &self.public_inputs,
            verifying_key.circuit_size,
            verifying_key.pub_inputs_offset,
        );
        *self.memory.witness_commitments.z_perm_mut() =
            transcript.receive_point_from_prover("z_perm".to_string())?;
        Ok(())
    }

    pub(crate) fn verify(
        mut self,
        verifying_key: &VerifyingKey,
        transcript: &mut Transcript<H>,
    ) -> HonkVerifyResult<VerifierMemory> {
        tracing::trace!("Oink verify");
        self.execute_preamble_round(verifying_key, transcript)?;
        self.execute_wire_commitments_round(transcript)?;
        self.execute_sorted_list_accumulator_round(transcript)?;
        self.execute_log_derivative_inverse_round(transcript)?;
        self.execute_grand_product_computation_round(verifying_key, transcript)?;
        Oink::<P, H>::generate_alphas_round(&mut self.memory.challenges.alphas, transcript);
        Ok(self.memory)
    }
}
