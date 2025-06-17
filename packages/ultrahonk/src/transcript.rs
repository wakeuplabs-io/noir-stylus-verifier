use crate::backends::HashBackend;
use crate::honk_curve::{NUM_BASEFIELD_ELEMENTS, NUM_SCALARFIELD_ELEMENTS};
use crate::serialize::{BytesDeserializable, BytesSerializable};
use crate::types::{G1Affine, HonkProof, HonkProofError, HonkProofResult, ScalarField};
use crate::Utils;
use alloc::vec::Vec;
use ark_ec::AffineRepr;
use ark_ff::{One, Zero};
use num_bigint::BigUint;
use std::{collections::BTreeMap, ops::Index};

pub struct Transcript<H>
where
    H: HashBackend,
{
    proof_data: Vec<ScalarField>,
    manifest: TranscriptManifest,
    num_frs_written: usize, // the number of bb::frs written to proof_data by the prover or the verifier
    num_frs_read: usize,    // the number of bb::frs read from proof_data by the verifier
    round_number: usize,
    is_first_challenge: bool,
    current_round_data: Vec<ScalarField>,
    previous_challenge: ScalarField,
    phantom_data: std::marker::PhantomData<H>,
}

impl<H> Default for Transcript<H>
where
    H: HashBackend,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<H> Transcript<H>
where
    H: HashBackend,
{
    pub fn new() -> Self {
        Self {
            proof_data: Default::default(),
            manifest: Default::default(),
            num_frs_written: 0,
            num_frs_read: 0,
            round_number: 0,
            is_first_challenge: true,
            current_round_data: Default::default(),
            previous_challenge: Default::default(),
            phantom_data: Default::default(),
        }
    }

    pub fn new_verifier(proof: HonkProof) -> Self {
        Self {
            proof_data: proof.inner(),
            manifest: Default::default(),
            num_frs_written: 0,
            num_frs_read: 0,
            round_number: 0,
            is_first_challenge: true,
            current_round_data: Default::default(),
            previous_challenge: Default::default(),
            phantom_data: Default::default(),
        }
    }

    pub fn get_proof(self) -> HonkProof {
        HonkProof::new(self.proof_data)
    }

    #[expect(dead_code)]
    pub(crate) fn print(&self) {
        self.manifest.print();
    }

    #[expect(dead_code)]
    pub(crate) fn get_manifest(&self) -> &TranscriptManifest {
        &self.manifest
    }

    fn add_element_frs_to_hash_buffer(&mut self, label: String, elements: &[ScalarField]) {
        // Add an entry to the current round of the manifest
        let len = elements.len();
        self.manifest.add_entry(self.round_number, label, len);
        self.current_round_data.extend(elements);
        self.num_frs_written += len;
    }

    // Adds an element to the transcript.
    // Serializes the element to frs and adds it to the current_round_data buffer. Does NOT add the element to the proof. This is used for elements which should be part of the transcript but are not in the final proof (e.g. circuit size)
    fn add_to_hash_buffer(&mut self, label: String, elements: &[ScalarField]) {
        self.add_element_frs_to_hash_buffer(label, elements);
    }

    fn send_to_verifier(&mut self, label: String, elements: &[ScalarField]) {
        self.proof_data.extend(elements);
        self.add_element_frs_to_hash_buffer(label, elements);
    }

    pub fn send_fr_to_verifier(&mut self, label: String, element: ScalarField) {
        let elements: Vec<ScalarField> = vec![element.to_owned()];
        self.send_to_verifier(label, &elements);
    }

    pub fn send_u64_to_verifier(&mut self, label: String, element: u64) {
        let el = ScalarField::from(element);
        self.send_to_verifier(label, &[el]);
    }

    pub fn add_u64_to_hash_buffer(&mut self, label: String, element: u64) {
        let el = ScalarField::from(element);
        self.add_to_hash_buffer(label, &[el]);
    }

    pub fn send_fr_iter_to_verifier<'a, I: IntoIterator<Item = &'a ScalarField>>(
        &mut self,
        label: String,
        element: I,
    ) {
        let elements: Vec<ScalarField> = element.into_iter().map(|src| src.to_owned()).collect();
        self.send_to_verifier(label, &elements);
    }

    fn receive_n_from_prover(
        &mut self,
        label: String,
        n: usize,
    ) -> HonkProofResult<Vec<ScalarField>> {
        if self.num_frs_read + n > self.proof_data.len() {
            return Err(HonkProofError::ProofTooSmall);
        }
        let elements = self.proof_data[self.num_frs_read..self.num_frs_read + n].to_owned();
        self.num_frs_read += n;

        self.add_element_frs_to_hash_buffer(label, &elements);
        Ok(elements)
    }

    pub(super) fn receive_fr_from_prover(&mut self, label: String) -> HonkProofResult<ScalarField> {
        let elements = self.receive_n_from_prover(label, NUM_SCALARFIELD_ELEMENTS)?;

        Ok(Utils::convert_scalarfield_back(&elements))
    }

    pub(super) fn receive_point_from_prover(&mut self, label: String) -> HonkProofResult<G1Affine> {
        let elements = self.receive_n_from_prover(label, NUM_BASEFIELD_ELEMENTS * 2)?;

        let coords = elements
            .chunks_exact(NUM_BASEFIELD_ELEMENTS)
            .map(Utils::convert_basefield_back)
            .collect::<Vec<_>>();

        let x = coords[0];
        let y = coords[1];

        let res = if x.is_zero() && y.is_zero() {
            G1Affine::zero()
        } else {
            G1Affine::new(x, y)
        };

        Ok(res)
    }

    pub(super) fn receive_fr_vec_from_verifier(
        &mut self,
        label: String,
        n: usize,
    ) -> HonkProofResult<Vec<ScalarField>> {
        let elements = self.receive_n_from_prover(label, NUM_SCALARFIELD_ELEMENTS * n)?;

        let elements = elements
            .chunks_exact(NUM_SCALARFIELD_ELEMENTS)
            .map(Utils::convert_scalarfield_back)
            .collect();

        Ok(elements)
    }

    pub(super) fn receive_fr_array_from_verifier<const SIZE: usize>(
        &mut self,
        label: String,
    ) -> HonkProofResult<[ScalarField; SIZE]> {
        let mut res: [ScalarField; SIZE] = [ScalarField::zero(); SIZE];
        let elements = self.receive_n_from_prover(label, NUM_SCALARFIELD_ELEMENTS * SIZE)?;

        for (src, des) in elements
            .chunks_exact(NUM_SCALARFIELD_ELEMENTS)
            .zip(res.iter_mut())
        {
            let el = Utils::convert_scalarfield_back(src);
            *des = el;
        }
        Ok(res)
    }

    fn split_challenge(challenge: ScalarField) -> [ScalarField; 2] {
        // match the parameter used in stdlib, which is derived from cycle_scalar (is 128)
        const LO_BITS: usize = 128;
        let biguint: BigUint = challenge.into();

        let lower_mask = (BigUint::one() << LO_BITS) - BigUint::one();
        let lo = &biguint & lower_mask;
        let hi = biguint >> LO_BITS;

        let lo = ScalarField::from(lo);
        let hi = ScalarField::from(hi);

        [lo, hi]
    }

    fn get_next_duplex_challenge_buffer(&mut self, num_challenges: usize) -> [ScalarField; 2] {
        // challenges need at least 110 bits in them to match the presumed security parameter of the BN254 curve.
        assert!(num_challenges <= 2);
        // Prevent challenge generation if this is the first challenge we're generating,
        // AND nothing was sent by the prover.
        if self.is_first_challenge {
            assert!(!self.current_round_data.is_empty());
        }
        // concatenate the previous challenge (if this is not the first challenge) with the current round data.
        // AZTEC TODO(Adrian): Do we want to use a domain separator as the initial challenge buffer?
        // We could be cheeky and use the hash of the manifest as domain separator, which would prevent us from having
        // to domain separate all the data. (See https://safe-hash.dev)

        let mut full_buffer = Vec::new();
        std::mem::swap(&mut full_buffer, &mut self.current_round_data);

        if self.is_first_challenge {
            // Update is_first_challenge for the future
            self.is_first_challenge = false;
        } else {
            // if not the first challenge, we can use the previous_challenge
            full_buffer.insert(0, self.previous_challenge);
        }

        // Hash the full buffer with poseidon2, which is believed to be a collision resistant hash function and a random
        // oracle, removing the need to pre-hash to compress and then hash with a random oracle, as we previously did
        // with Pedersen and Blake3s.
        let new_challenge_bytes = H::hash(full_buffer.serialize_to_bytes());
        let new_challenge = ScalarField::deserialize_from_bytes(&new_challenge_bytes).unwrap();
        let new_challenges = Self::split_challenge(new_challenge);

        // update previous challenge buffer for next time we call this function
        self.previous_challenge = new_challenge;
        new_challenges
    }

    pub fn get_challenge(&mut self, label: String) -> ScalarField {
        self.manifest.add_challenge(self.round_number, &[label]);
        let challenge = self.get_next_duplex_challenge_buffer(1)[0];
        let res = challenge.to_owned();
        self.round_number += 1;
        res
    }

    pub fn get_challenges(&mut self, labels: &[String]) -> Vec<ScalarField> {
        let num_challenges = labels.len();
        self.manifest.add_challenge(self.round_number, labels);

        let mut res = Vec::with_capacity(num_challenges);
        for _ in 0..num_challenges >> 1 {
            let challenge_buffer = self.get_next_duplex_challenge_buffer(2);
            res.push(challenge_buffer[0].to_owned());
            res.push(challenge_buffer[1].to_owned());
        }
        if num_challenges & 1 == 1 {
            let challenge_buffer = self.get_next_duplex_challenge_buffer(1);
            res.push(challenge_buffer[0].to_owned());
        }

        self.round_number += 1;
        res
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub(crate) struct RoundData {
    challenge_label: Vec<String>,
    entries: Vec<(String, usize)>,
}

impl RoundData {
    pub(crate) fn print(&self) {
        for label in self.challenge_label.iter() {
            println!("\tchallenge: {}", label);
        }
        for entry in self.entries.iter() {
            println!("\telement ({}): {}", entry.1, entry.0);
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TranscriptManifest {
    manifest: BTreeMap<usize, RoundData>,
}

impl TranscriptManifest {
    pub(crate) fn print(&self) {
        for round in self.manifest.iter() {
            println!("Round: {}", round.0);
            round.1.print();
        }
    }

    pub(crate) fn add_challenge(&mut self, round: usize, labels: &[String]) {
        self.manifest
            .entry(round)
            .or_default()
            .challenge_label
            .extend_from_slice(labels);
    }

    pub(crate) fn add_entry(&mut self, round: usize, element_label: String, element_size: usize) {
        self.manifest
            .entry(round)
            .or_default()
            .entries
            .push((element_label, element_size));
    }

    #[expect(dead_code)]
    pub(crate) fn size(&self) -> usize {
        self.manifest.len()
    }
}

impl Index<usize> for TranscriptManifest {
    type Output = RoundData;

    fn index(&self, index: usize) -> &Self::Output {
        &self.manifest[&index]
    }
}
