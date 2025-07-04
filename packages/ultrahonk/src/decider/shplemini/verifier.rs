use super::{
    types::{PolyF, PolyG, PolyGShift},
    ShpleminiVerifierOpeningClaim,
};
use crate::{alloc::string::ToString, backends::G1ArithmeticBackend, constants::get_crs_g2, types::HonkProofError};
use crate::{
    backends::HashBackend,
    decider::verifier::DeciderVerifier,
    transcript::Transcript,
    types::{G1Affine, ScalarField},
    verifier::HonkVerifyResult,
    CONST_PROOF_SIZE_LOG_N, NUM_INTERLEAVING_CLAIMS,
};
use alloc::vec::Vec;
use ark_bn254::G2Affine;
use ark_ec::AffineRepr;
use ark_ff::{Field, One, Zero};

impl<P: G1ArithmeticBackend, H: HashBackend> DeciderVerifier<P, H> {
    pub fn verify_shplemini(
        &mut self,
        transcript: &mut Transcript,
        multivariate_challenge: Vec<ScalarField>,
        circuit_size: u32,
    ) -> HonkVerifyResult<bool> {
        let log_circuit_size = circuit_size.ilog2() as usize;

        let mut padding_indicator_array = [ScalarField::zero(); CONST_PROOF_SIZE_LOG_N];

        for (idx, value) in padding_indicator_array.iter_mut().enumerate() {
            *value = if idx < log_circuit_size as usize {
                ScalarField::one()
            } else {
                ScalarField::zero()
            };
        }

        let mut opening_claim = self.compute_batch_opening_claim(
            multivariate_challenge,
            transcript,
            &padding_indicator_array,
        )?;

        let pairing_points = Self::reduce_verify_shplemini(&mut opening_claim, transcript)?;
        // let pairing_points = (G1Affine::zero(), G1Affine::zero());

        let pcs_verified = P::ec_pairing_check(
            pairing_points.0,
            pairing_points.1,
            get_crs_g2(),
            G2Affine::generator(),
        )
        .unwrap();

        Ok(pcs_verified)
    }

    fn reduce_verify_shplemini(
        opening_pair: &mut ShpleminiVerifierOpeningClaim,
        transcript: &mut Transcript,
    ) -> HonkVerifyResult<(G1Affine, G1Affine)> {
        let quotient_commitment = transcript.receive_point_from_prover("KZG:W".to_string())?;
        opening_pair.commitments.push(quotient_commitment);
        opening_pair.scalars.push(opening_pair.challenge);
        let p_1 = -quotient_commitment.into_group();

        let p_0 = P::msm(&opening_pair.scalars, &opening_pair.commitments)
            .map_err(|_| HonkProofError::MSMError)?;

        Ok((p_0, p_1.into()))
    }

    pub fn powers_of_evaluation_challenge(
        gemini_evaluation_challenge: ScalarField,
        num_squares: usize,
    ) -> Vec<ScalarField> {
        let mut squares = Vec::with_capacity(num_squares);
        squares.push(gemini_evaluation_challenge);
        for j in 1..num_squares {
            squares.push(squares[j - 1].square());
        }
        squares
    }

    fn compute_inverted_gemini_denominators(
        shplonk_eval_challenge: &ScalarField,
        gemini_eval_challenge_powers: &[ScalarField],
    ) -> Vec<ScalarField> {
        let virtual_log_n = gemini_eval_challenge_powers.len();
        let num_gemini_claims = 2 * virtual_log_n;
        let mut denominators = Vec::with_capacity(num_gemini_claims);
        for gemini_eval_challenge_power in gemini_eval_challenge_powers {
            // Place 1/(z - r ^ {2^j})
            denominators.push(*shplonk_eval_challenge - *gemini_eval_challenge_power);
            // Place 1/(z + r ^ {2^j})
            denominators.push(*shplonk_eval_challenge + *gemini_eval_challenge_power);
        }

        crate::Utils::batch_invert(&mut denominators);

        denominators
    }

    pub fn compute_batch_opening_claim(
        &self,
        multivariate_challenge: Vec<ScalarField>,
        transcript: &mut Transcript,
        padding_indicator_array: &[ScalarField; CONST_PROOF_SIZE_LOG_N],
        // const core::vector<RefVector<Commitment>>& concatenation_group_commitments = {},
        // RefSpan<ScalarField> concatenated_evaluations = {}
    ) -> HonkVerifyResult<ShpleminiVerifierOpeningClaim> {
        let virtual_log_n = multivariate_challenge.len();

        let mut batched_evaluation = ScalarField::zero();

        // Get the challenge ρ to batch commitments to multilinear polynomials and their shifts
        let gemini_batching_challenge = transcript.get_challenge::<H>("rho".to_string());

        // Process Gemini transcript data:
        // - Get Gemini commitments (com(A₁), com(A₂), … , com(Aₙ₋₁))
        let fold_commitments: Vec<_> = (0..virtual_log_n - 1)
            .map(|i| transcript.receive_point_from_prover(format!("Gemini:FOLD_{}", i + 1)))
            .collect::<Result<_, _>>()?;

        // - Get Gemini evaluation challenge for Aᵢ, i = 0, … , d−1
        let gemini_evaluation_challenge = transcript.get_challenge::<H>("Gemini:r".to_string());

        // - Get evaluations (A₀(−r), A₁(−r²), ... , Aₙ₋₁(−r²⁽ⁿ⁻¹⁾))
        let gemini_fold_neg_evaluations: Vec<_> = (1..=virtual_log_n)
            .map(|i| transcript.receive_fr_from_prover(format!("Gemini:a_{}", i + 1)))
            .collect::<Result<_, _>>()?;

        // Get evaluations of partially evaluated batched interleaved polynomials P₊(rˢ) and P₋((-r)ˢ)
        let p_pos = ScalarField::zero();
        let p_neg = ScalarField::zero();
        // if (claim_batcher.interleaved) {
        //     p_pos = transcript->template receive_from_prover<ScalarField>("Gemini:P_pos");
        //     p_neg = transcript->template receive_from_prover<ScalarField>("Gemini:P_neg");
        // }

        // - Compute vector (r, r², ... , r^{2^{d-1}}), where d = log_n
        let gemini_eval_challenge_powers =
            Self::powers_of_evaluation_challenge(gemini_evaluation_challenge, virtual_log_n);

        // Process Shplonk transcript data:
        // - Get Shplonk batching challenge
        let shplonk_batching_challenge = transcript.get_challenge::<H>("Shplonk:nu".to_string());

        // Compute the powers of ν that are required for batching Gemini, SmallSubgroupIPA, and committed sumcheck
        // univariate opening claims.
        let shplonk_batching_challenge_powers = Self::compute_shplonk_batching_challenge_powers(
            shplonk_batching_challenge,
            virtual_log_n,
        );

        // - Get the quotient commitment for the Shplonk batching of Gemini opening claims
        let q_commitment = transcript.receive_point_from_prover("Shplonk:Q".to_string())?;

        // Start populating the vector (Q, f₀, ... , fₖ₋₁, g₀, ... , gₘ₋₁, com(A₁), ... , com(Aₙ₋₁), [1]₁) where fᵢ are
        // the k commitments to unshifted polynomials and gⱼ are the m commitments to shifted polynomials

        // Get Shplonk opening point z
        let shplonk_evaluation_challenge = transcript.get_challenge::<H>("Shplonk:z".to_string());

        // Start computing the scalar to be multiplied by [1]₁
        let mut constant_term_accumulator = ScalarField::zero();

        let mut opening_claim: ShpleminiVerifierOpeningClaim = ShpleminiVerifierOpeningClaim {
            challenge: shplonk_evaluation_challenge,
            scalars: Vec::new(),
            commitments: vec![q_commitment],
        };
        opening_claim.scalars.push(ScalarField::one());

        // Compute 1/(z − r), 1/(z + r), 1/(z - r²),  1/(z + r²), … , 1/(z - r^{2^{d-1}}), 1/(z + r^{2^{d-1}})
        // These represent the denominators of the summand terms in Shplonk partially evaluated polynomial Q_z
        let inverse_vanishing_evals: Vec<ScalarField> = Self::compute_inverted_gemini_denominators(
            &opening_claim.challenge,
            &gemini_eval_challenge_powers,
        );

        // TACEO NOTE: so far we have no interleaved polynomials so some parts here are skipped

        // Compute the additional factors to be multiplied with unshifted and shifted commitments when lazily
        // reconstructing the commitment of Q_z
        // i-th unshifted commitment is multiplied by −ρⁱ and the unshifted_scalar ( 1/(z−r) + ν/(z+r) )
        let unshifted_scalar =
            inverse_vanishing_evals[0] + shplonk_batching_challenge * inverse_vanishing_evals[1];

        // j-th shifted commitment is multiplied by −ρᵏ⁺ʲ⁻¹ and the shifted_scalar r⁻¹ ⋅ (1/(z−r) − ν/(z+r))
        let shifted_scalar = gemini_evaluation_challenge.inverse().unwrap()
            * (inverse_vanishing_evals[0]
                - shplonk_batching_challenge * inverse_vanishing_evals[1]);

        let gemini_batching_challenge_power = ScalarField::one();

        // Append the commitments and scalars from each batch of claims to the Shplemini, vectors which subsequently
        // will be inputs to the batch mul;
        // update the batched evaluation and the running batching challenge (power of rho) in place.
        // Update the commitments and scalars vectors as well as the batched evaluation given the present batches
        self.update_batch_mul_inputs_and_batched_evaluation(
            &gemini_batching_challenge,
            &unshifted_scalar,
            &shifted_scalar,
            &mut opening_claim,
            &mut batched_evaluation,
            &gemini_batching_challenge_power,
        );

        // Reconstruct Aᵢ(r²ⁱ) for i=0, ..., n-1 from the batched evaluation of the multilinear polynomials and Aᵢ(−r²ⁱ)
        // for i = 0, ..., n-1.
        // In the case of interleaving, we compute A₀(r) as A₀₊(r) + P₊(r^s).
        let gemini_fold_pos_evaluations = Self::compute_fold_pos_evaluations(
            padding_indicator_array,
            &batched_evaluation,
            &multivariate_challenge,
            &gemini_eval_challenge_powers,
            &gemini_fold_neg_evaluations,
            p_neg,
        );

        // Place the commitments to Gemini fold polynomials Aᵢ in the vector of batch_mul commitments, compute the
        // contributions from Aᵢ(−r²ⁱ) for i=1, … , n−1 to the constant term accumulator, add corresponding scalars for
        // the batch mul
        Self::batch_gemini_claims_received_from_prover(
            padding_indicator_array,
            &fold_commitments,
            &gemini_fold_neg_evaluations,
            &gemini_fold_pos_evaluations,
            &inverse_vanishing_evals,
            &shplonk_batching_challenge_powers,
            &mut opening_claim,
            &mut constant_term_accumulator,
        );

        let full_a_0_pos = gemini_fold_pos_evaluations[0];

        // Retrieve  the contribution without P₊(r^s)
        let a_0_pos = full_a_0_pos - p_pos;
        // Add contributions from A₀₊(r) and  A₀₋(-r) to constant_term_accumulator:
        //  Add  A₀₊(r)/(z−r) to the constant term accumulator
        constant_term_accumulator += a_0_pos * inverse_vanishing_evals[0];
        // Add  A₀₋(-r)/(z+r) to the constant term accumulator
        constant_term_accumulator += gemini_fold_neg_evaluations[0]
            * shplonk_batching_challenge
            * inverse_vanishing_evals[1];

        // TACEO TODO:
        // // - Add A₀(r)/(z−r) to the constant term accumulator
        // constant_term_accumulator += a_0_pos * inverse_vanishing_evals[0];
        // // Add A₀(−r)/(z+r) to the constant term accumulator
        // constant_term_accumulator += gemini_fold_neg_evaluations[0]
        //     * shplonk_batching_challenge
        //     * inverse_vanishing_evals[1];

        // TACEO TODO: BB removes repeated commitments here to reduce the number of scalar muls
        // remove_repeated_commitments(commitments, scalars, repeated_commitments, has_zk);

        // Finalize the batch opening claim
        opening_claim.commitments.push(G1Affine::generator());
        opening_claim.scalars.push(constant_term_accumulator);
        Ok(opening_claim)
    }

    /**
     * @brief Append the commitments and scalars from each batch of claims to the Shplemini, vectors which subsequently
     * will be inputs to the batch mul;
     * update the batched evaluation and the running batching challenge (power of rho) in place.
     *
     * @param commitments commitment inputs to the single Shplemini batch mul
     * @param scalars scalar inputs to the single Shplemini batch mul
     * @param batched_evaluation running batched evaluation of the committed multilinear polynomials
     * @param rho multivariate batching challenge \rho
     * @param rho_power current power of \rho used in the batching scalar
     * @param shplonk_batching_pos and @param shplonk_batching_neg consecutive powers of the Shplonk batching
     * challenge ν for the interleaved contributions
     */
    fn update_batch_mul_inputs_and_batched_evaluation(
        &self,
        multivariate_batching_challenge: &ScalarField,
        unshifted_scalar: &ScalarField,
        shifted_scalar: &ScalarField,
        opening_claim: &mut ShpleminiVerifierOpeningClaim,
        batched_evaluation: &mut ScalarField,
        gemini_batching_challenge_power: &ScalarField,
    ) {
        let mut current_batching_challenge = *gemini_batching_challenge_power;
        let unshifted_evaluations = PolyF::from(&self.memory.claimed_evaluations);
        let shifted_evaluations = PolyGShift::from(&self.memory.claimed_evaluations);
        let unshifted_commitments = PolyF::from(&self.memory.verifier_commitments);
        let to_be_shifted_commitments = PolyG::from(&self.memory.verifier_commitments);
        for (unshifted_commitment, unshifted_evaluation) in unshifted_commitments
            .iter()
            .zip(unshifted_evaluations.iter())
        {
            // Move unshifted commitments to the 'commitments' vector
            opening_claim.commitments.push(*unshifted_commitment);
            // Compute −ρⁱ ⋅ (1/(z−r) + ν/(z+r)) and place into 'scalars'
            opening_claim
                .scalars
                .push(-(*unshifted_scalar) * current_batching_challenge);
            // Accumulate the evaluation of ∑ ρⁱ ⋅ fᵢ at the sumcheck challenge
            *batched_evaluation += *unshifted_evaluation * current_batching_challenge;
            // Update the batching challenge
            current_batching_challenge *= *multivariate_batching_challenge;
        }
        for (shifted_commitment, shifted_evaluation) in to_be_shifted_commitments
            .iter()
            .zip(shifted_evaluations.iter())
        {
            // Move shifted commitments to the 'commitments' vector
            opening_claim.commitments.push(*shifted_commitment);
            // Compute −ρ⁽ᵏ⁺ʲ⁾ ⋅ r⁻¹ ⋅ (1/(z−r) − ν/(z+r)) and place into 'scalars'
            opening_claim
                .scalars
                .push(-(*shifted_scalar) * current_batching_challenge);
            // Accumulate the evaluation of ∑ ρ⁽ᵏ⁺ʲ⁾ ⋅ f_shift at the sumcheck challenge
            *batched_evaluation += *shifted_evaluation * current_batching_challenge;
            // Update the batching challenge ρ
            current_batching_challenge *= *multivariate_batching_challenge;
        }
    }

    /**
     * @brief Populates the 'commitments' and 'scalars' vectors with the commitments to Gemini fold polynomials \f$
     * A_i \f$.
     *
     * @details Once the commitments to Gemini "fold" polynomials \f$ A_i \f$ and their evaluations at \f$ -r^{2^i}
     * \f$, where \f$ i = 1, \ldots, n-1 \f$, are received by the verifier, it performs the following operations:
     *
     * 1. Moves the vector
     *    \f[
     *    \left( \text{com}(A_1), \text{com}(A_2), \ldots, \text{com}(A_{n-1}) \right)
     *    \f]
     *    to the 'commitments' vector.
     *
     * 2. Computes the scalars:
     *    \f[
     *    \frac{\nu^{2}}{z + r^2}, \frac{\nu^3}{z + r^4}, \ldots, \frac{\nu^{n-1}}{z + r^{2^{n-1}}}
     *    \f]
     *    and places them into the 'scalars' vector.
     *
     * 3. Accumulates the summands of the constant term:
     *    \f[
     *    \sum_{i=2}^{n-1} \frac{\nu^{i} \cdot A_i(-r^{2^i})}{z + r^{2^i}}
     *    \f]
     *    and adds them to the 'constant_term_accumulator'.
     *
     * @param log_circuit_size The logarithm of the circuit size, determining the depth of the Gemini protocol.
     * @param fold_commitments A vector containing the commitments to the Gemini fold polynomials \f$ A_i \f$.
     * @param gemini_evaluations A vector containing the evaluations of the Gemini fold polynomials \f$ A_i \f$ at
     * points \f$ -r^{2^i} \f$.
     * @param inverse_vanishing_evals A vector containing the inverse evaluations of the vanishing polynomial.
     * @param shplonk_batching_challenge The batching challenge \f$ \nu \f$ used in the SHPLONK protocol.
     * @param commitments Output vector where the commitments to the Gemini fold polynomials will be stored.
     * @param scalars Output vector where the computed scalars will be stored.
     * @param constant_term_accumulator The accumulator for the summands of the constant term.
     */
    #[expect(clippy::too_many_arguments)]
    fn batch_gemini_claims_received_from_prover(
        padding_indicator_array: &[ScalarField; CONST_PROOF_SIZE_LOG_N],
        fold_commitments: &[G1Affine],
        gemini_neg_evaluations: &[ScalarField],
        gemini_pos_evaluations: &[ScalarField],
        inverse_vanishing_evals: &[ScalarField],
        shplonk_batching_challenge_powers: &[ScalarField],
        opening_claim: &mut ShpleminiVerifierOpeningClaim,
        constant_term_accumulator: &mut ScalarField,
    ) {
        let virtual_log_n = gemini_neg_evaluations.len();
        // Start from 1, because the commitment to A_0 is reconstructed from the commitments to the multilinear
        // polynomials. The corresponding evaluations are also handled separately.
        for j in 1..virtual_log_n {
            // The index of 1/ (z - r^{2^{j}}) in the vector of inverted Gemini denominators
            let pos_index = 2 * j;
            // The index of 1/ (z + r^{2^{j}}) in the vector of inverted Gemini denominators
            let neg_index = 2 * j + 1;

            // Compute the "positive" scaling factor  (ν^{2j}) / (z - r^{2^{j}})
            let scaling_factor_pos =
                shplonk_batching_challenge_powers[pos_index] * inverse_vanishing_evals[pos_index];
            // Compute the "negative" scaling factor  (ν^{2j+1}) / (z + r^{2^{j}})
            let scaling_factor_neg =
                shplonk_batching_challenge_powers[neg_index] * inverse_vanishing_evals[neg_index];

            // Accumulate the const term contribution given by
            // v^{2j} * A_j(r^{2^j}) /(z - r^{2^j}) + v^{2j+1} * A_j(-r^{2^j}) /(z+ r^{2^j})
            *constant_term_accumulator += scaling_factor_neg * gemini_neg_evaluations[j]
                + scaling_factor_pos * gemini_pos_evaluations[j];

            // Place the scaling factor to the 'scalars' vector
            opening_claim
                .scalars
                .push(-padding_indicator_array[j] * (scaling_factor_neg + scaling_factor_pos));
            // Move com(Aᵢ) to the 'commitments' vector
            opening_claim.commitments.push(fold_commitments[j - 1]);
        }
    }

    /**
     * @brief Compute \f$ A_0(r), A_1(r^2), \ldots, A_{d-1}(r^{2^{d-1}})\f$
     *
     * Recall that \f$ A_0(r) = \sum \rho^i \cdot f_i + \frac{1}{r} \cdot \sum \rho^{i+k} g_i \f$, where \f$
     * k \f$ is the number of "unshifted" commitments.
     *
     * @details Initialize `a_pos` = \f$ A_{d}(r) \f$ with the batched evaluation \f$ \sum \rho^i f_i(\vec{u}) + \sum
     * \rho^{i+k} g_i(\vec{u}) \f$. The verifier recovers \f$ A_{l-1}(r^{2^{l-1}}) \f$ from the "negative" value \f$
     * A_{l-1}\left(-r^{2^{l-1}}\right) \f$ received from the prover and the value \f$ A_{l}\left(r^{2^{l}}\right) \f$
     * computed at the previous step. Namely, the verifier computes
     * \f{align}{ A_{l-1}\left(r^{2^{l-1}}\right) =
     * \frac{2 \cdot r^{2^{l-1}} \cdot A_{l}\left(r^{2^l}\right) - A_{l-1}\left( -r^{2^{l-1}} \right)\cdot
     * \left(r^{2^{l-1}} (1-u_{l-1}) - u_{l-1}\right)} {r^{2^{l-1}} (1- u_{l-1}) + u_{l-1}}. \f}
     *
     * In the case of interleaving, the first "negative" evaluation has to be corrected by the contribution from \f$
     * P_{-}(-r^s)\f$, where \f$ s \f$ is the size of the group to be interleaved.
     *
     * @param batched_evaluation The evaluation of the batched polynomial at \f$ (u_0, \ldots, u_{d-1})\f$.
     * @param evaluation_point Evaluation point \f$ (u_0, \ldots, u_{d-1}) \f$ padded to CONST_PROOF_SIZE_LOG_N.
     * @param challenge_powers Powers of \f$ r \f$, \f$ r^2 \), ..., \( r^{2^{d-1}} \f$.
     * @param fold_neg_evals  Evaluations \f$ A_{i-1}(-r^{2^{i-1}}) \f$.
     * @return Evaluation \f$ A_0(r) \f$.
     */
    pub fn compute_fold_pos_evaluations(
        padding_indicator_array: &[ScalarField; CONST_PROOF_SIZE_LOG_N],
        batched_evaluation: &ScalarField,
        evaluation_point: &[ScalarField], // CONST_PROOF_SIZE
        challenge_powers: &[ScalarField], // r_squares CONST_PROOF_SIZE_LOG_N
        fold_neg_evals: &[ScalarField],
        p_neg: ScalarField,
    ) -> Vec<ScalarField> {
        let virtual_log_n = evaluation_point.len();

        let mut evals = fold_neg_evals.to_vec();

        let mut eval_pos_prev = *batched_evaluation;

        let mut fold_pos_evaluations = Vec::with_capacity(virtual_log_n);
        // Either a computed eval of A_i at r^{2^i}, or 0
        let mut value_to_emplace;

        // Add the contribution of P-((-r)ˢ) to get A_0(-r), which is 0 if there are no interleaved polynomials
        evals[0] += p_neg;

        // Solve the sequence of linear equations
        for l in (1..=virtual_log_n).rev() {
            // Get r²⁽ˡ⁻¹⁾
            let challenge_power = challenge_powers[l - 1];
            // Get uₗ₋₁
            let u = evaluation_point[l - 1];
            let eval_neg = evals[l - 1];
            // Get A₍ₗ₋₁₎(−r²⁽ˡ⁻¹⁾)
            // Compute the numerator
            let mut eval_pos = (challenge_power * eval_pos_prev * ScalarField::from(2u64))
                - eval_neg * (challenge_power * (ScalarField::one() - u) - u);
            // Divide by the denominator
            eval_pos *= (challenge_power * (ScalarField::one() - u) + u)
                .inverse()
                .expect("Non-zero denominator");

            // If current index is bigger than log_n, we propagate `batched_evaluation` to the next
            // round. Otherwise, current `eval_pos` A₍ₗ₋₁₎(−r²⁽ˡ⁻¹⁾) becomes `eval_pos_prev` in the round l-2.
            eval_pos_prev = padding_indicator_array[l - 1] * eval_pos
                + (ScalarField::one() - padding_indicator_array[l - 1]) * eval_pos_prev;
            // If current index is bigger than log_n, we emplace 0, which is later multiplied against
            // Commitment::one().
            value_to_emplace = padding_indicator_array[l - 1] * eval_pos_prev;
            fold_pos_evaluations.push(value_to_emplace);
        }

        fold_pos_evaluations.reverse();

        fold_pos_evaluations
    }

    /**
     * @brief A helper used by Shplemini Verifier. Precomputes a vector of the powers of \f$ \nu \f$ needed to batch all
     * univariate claims.
     *
     */
    fn compute_shplonk_batching_challenge_powers(
        shplonk_batching_challenge: ScalarField,
        virtual_log_n: usize,
        // committed_sumcheck: bool, we don't have this (yet)
    ) -> Vec<ScalarField> {
        let num_powers = 2 * virtual_log_n + NUM_INTERLEAVING_CLAIMS as usize;
        // // Each round univariate is opened at 0, 1, and a round challenge.
        // const NUM_COMMITTED_SUMCHECK_CLAIMS_PER_ROUND: usize = 3;

        // if committed_sumcheck {
        //     num_powers += NUM_COMMITTED_SUMCHECK_CLAIMS_PER_ROUND * CONST_PROOF_SIZE_LOG_N;
        // }

        let mut result = Vec::with_capacity(num_powers);
        result.push(ScalarField::one());
        for idx in 1..num_powers {
            result.push(result[idx - 1] * shplonk_batching_challenge);
        }
        result
    }
}
