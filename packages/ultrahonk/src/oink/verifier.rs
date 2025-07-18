use super::types::VerifierMemory;
use crate::{
    backends::HashBackend,
    transcript::Transcript,
    types::{HonkVerifyResult, ScalarField, VerifyingKey},
    NUM_ALPHAS,
};
use alloc::vec::Vec;
use ark_ff::One;

#[derive(Default)]
pub struct OinkVerifier {
    memory: VerifierMemory,
    pub(crate) public_inputs: Vec<ScalarField>,
}

impl OinkVerifier {
    /// Oink Verifier function that runs all the rounds of the verifier
    pub fn verify<H: HashBackend>(
        mut self,
        verifying_key: &VerifyingKey,
        transcript: &mut Transcript,
    ) -> HonkVerifyResult<VerifierMemory> {
        self.execute_preamble_round(verifying_key, transcript)?;
        self.execute_wire_commitments_round(transcript)?;
        self.execute_sorted_list_accumulator_round::<H>(transcript)?;
        self.execute_log_derivative_inverse_round::<H>(transcript)?;
        self.execute_grand_product_computation_round(verifying_key, transcript)?;
        Self::generate_alphas_round::<H>(&mut self.memory.challenges.alphas, transcript);
        Ok(self.memory)
    }

    /// Get circuit size, public input size, and public inputs from transcript
    fn execute_preamble_round(
        &mut self,
        verifying_key: &VerifyingKey,
        transcript: &mut Transcript,
    ) -> HonkVerifyResult<()> {
        let circuit_size = verifying_key.circuit_size as u64;
        let public_input_size = verifying_key.num_public_inputs as u64;
        let pub_inputs_offset = verifying_key.pub_inputs_offset as u64;

        transcript.add_u64_to_hash_buffer(circuit_size); // "circuit_size"
        transcript.add_u64_to_hash_buffer(public_input_size); // "public_input_size"
        transcript.add_u64_to_hash_buffer(pub_inputs_offset); // "pub_inputs_offset"

        self.public_inputs = Vec::with_capacity(public_input_size as usize);

        for _ in 0..public_input_size {
            let public_input = transcript.receive_fr_from_prover()?; // "public_input_{i}"
            self.public_inputs.push(public_input);
        }

        Ok(())
    }

    /// Get the wire polynomials (part of the witness), with the exception of the fourth wire, which is
    /// only received after adding memory records.
    fn execute_wire_commitments_round(
        &mut self,
        transcript: &mut Transcript,
    ) -> HonkVerifyResult<()> {
        *self.memory.witness_commitments.w_l_mut() = transcript.receive_point_from_prover()?; // "W_L"
        *self.memory.witness_commitments.w_r_mut() = transcript.receive_point_from_prover()?; // "W_R"
        *self.memory.witness_commitments.w_o_mut() = transcript.receive_point_from_prover()?; // "W_O"

        // Round is skipped since ultra_honk is no goblin flavor
        Ok(())
    }

    /// Get sorted witness-table accumulator and fourth wire commitments
    fn execute_sorted_list_accumulator_round<H: HashBackend>(
        &mut self,
        transcript: &mut Transcript,
    ) -> HonkVerifyResult<()> {
        let challs = transcript.get_challenges::<H>(3); // eta, eta_two, eta_three
        self.memory.challenges.eta_1 = challs[0];
        self.memory.challenges.eta_2 = challs[1];
        self.memory.challenges.eta_3 = challs[2];

        *self.memory.witness_commitments.lookup_read_counts_mut() =
            transcript.receive_point_from_prover()?; // "lookup_read_counts"

        *self.memory.witness_commitments.lookup_read_tags_mut() =
            transcript.receive_point_from_prover()?; // "lookup_read_tags"

        *self.memory.witness_commitments.w_4_mut() = transcript.receive_point_from_prover()?; // "w_4"

        Ok(())
    }

    /// Get log derivative inverse polynomial
    fn execute_log_derivative_inverse_round<H: HashBackend>(
        &mut self,
        transcript: &mut Transcript,
    ) -> HonkVerifyResult<()> {
        let challs = transcript.get_challenges::<H>(2); // beta, gamma
        self.memory.challenges.beta = challs[0];
        self.memory.challenges.gamma = challs[1];

        *self.memory.witness_commitments.lookup_inverses_mut() =
            transcript.receive_point_from_prover()?; // "lookup_inverses"

        // Round is done since ultra_honk is no goblin flavor
        Ok(())
    }

    /// Compute lookup grand product delta and get permutation and lookup grand product commitments
    fn execute_grand_product_computation_round(
        &mut self,
        verifying_key: &VerifyingKey,
        transcript: &mut Transcript,
    ) -> HonkVerifyResult<()> {
        self.memory.public_input_delta = Self::compute_public_input_delta(
            &self.memory.challenges.beta,
            &self.memory.challenges.gamma,
            &self.public_inputs,
            verifying_key.circuit_size,
            verifying_key.pub_inputs_offset,
        );
        *self.memory.witness_commitments.z_perm_mut() = transcript.receive_point_from_prover()?; // "z_perm"
        Ok(())
    }

    /// Compute lookup grand product delta and get permutation and lookup grand product commitments
    fn compute_public_input_delta(
        beta: &ScalarField,
        gamma: &ScalarField,
        public_inputs: &[ScalarField],
        circuit_size: u32,
        pub_inputs_offset: u32,
    ) -> ScalarField {
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
    fn generate_alphas_round<H: HashBackend>(
        alphas: &mut [ScalarField; NUM_ALPHAS],
        transcript: &mut Transcript,
    ) {
        alphas.copy_from_slice(&transcript.get_challenges::<H>(NUM_ALPHAS)); // alpha_{i from 0 to NUM_ALPHAS-1}
    }
}
