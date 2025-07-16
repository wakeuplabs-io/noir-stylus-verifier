use crate::alloc::borrow::ToOwned;
use crate::constants::NUM_U64S_FELT;
use alloc::vec::Vec;
use ark_bn254::{g1::Config as G1Config, g2::Config as G2Config, Fq, Fq2, Fr};
use ark_ec::short_weierstrass::Affine;
use ark_ff::{Fp256, MontBackend};

/// Type alias for an element of the scalar field of the Bn254 curve
pub type ScalarField = Fr;

/// Type alias for an element of the Bn254 curve's G1 pairing group
pub type G1Affine = Affine<G1Config>;

/// Type alias for an element of the Bn254 curve's G2 pairing group
pub type G2Affine = Affine<G2Config>;

/// Type alias for an element of the Bn254 curve's G1 pairing group's base field
pub type G1BaseField = Fq;

/// Type alias for an element of the Bn254 curve's G2 pairing group's base field
pub type G2BaseField = Fq2;

/// Type alias for the G1 group of the curve
pub type G1Projective = ark_bn254::G1Projective;

/// Type alias for a 256-bit prime field element in Montgomery form
pub type MontFp256<P> = Fp256<MontBackend<P, NUM_U64S_FELT>>;

#[derive(Debug)]
pub struct HonkProof {
    proof: Vec<ScalarField>,
}

pub(crate) type HonkVerifyResult<T> = Result<T, HonkProofError>;

/// The errors that may arise during the computation of a HONK proof.
#[derive(Debug, thiserror::Error)]
pub enum HonkProofError {
    /// Indicates that the witness is too small for the provided circuit.
    #[error("Cannot index into witness {0}")]
    CorruptedWitness(usize),
    /// The proof has too few elements
    #[error("Proof too small")]
    ProofTooSmall,
    /// Invalid proof length
    #[error("Invalid proof length")]
    InvalidProofLength,
    /// Invalid key length
    #[error("Invalid key length")]
    InvalidKeyLength,
    /// Corrupted Key
    #[error("Corrupted Key")]
    CorruptedKey,
    /// Expected Public Witness, Shared received
    #[error("Expected Public Witness, Shared received")]
    ExpectedPublicWitness,
    /// Gemini evaluation challenge is in the SmallSubgroup
    #[error("Gemini evaluation challenge is in the SmallSubgroup.")]
    GeminiSmallSubgroup,
    /// The Subgroup for the FFT domain is too large
    #[error("Too large Subgroup")]
    LargeSubgroup,
    /// Failed to deserialize G2Affine from transcript file
    #[error("Failed to deserialize G2Affine from file")]
    DeserializationError(),
    /// MSM error
    #[error("MSM error")]
    MSMError,
}

impl HonkProof {
    pub(crate) fn new(proof: Vec<ScalarField>) -> Self {
        Self { proof }
    }

    pub fn inner(self) -> Vec<ScalarField> {
        self.proof
    }

    pub fn separate_proof_and_public_inputs(
        self,
        num_public_inputs: usize,
    ) -> (Self, Vec<ScalarField>) {
        let (public_inputs, proof) = self.proof.split_at(num_public_inputs);
        (Self::new(proof.to_vec()), public_inputs.to_vec())
    }

    pub fn insert_public_inputs(self, public_inputs: Vec<ScalarField>) -> Self {
        let mut proof = public_inputs;
        proof.extend(self.proof.to_owned());
        Self::new(proof)
    }
}

pub(crate) const NUM_ALL_ENTITIES: usize =
    WITNESS_ENTITIES_SIZE + PRECOMPUTED_ENTITIES_SIZE + SHIFTED_WITNESS_ENTITIES_SIZE;

#[derive(Default, Clone)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct AllEntities<T: Default> {
    pub witness: WitnessEntities<T>,
    pub precomputed: PrecomputedEntities<T>,
    pub shifted_witness: ShiftedWitnessEntities<T>,
}

impl<T: Default> AllEntities<T> {
    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.precomputed
            .iter_mut()
            .chain(self.witness.iter_mut())
            .chain(self.shifted_witness.iter_mut())
    }
}

pub(crate) const WITNESS_ENTITIES_SIZE: usize = 8;

#[derive(Default, Clone)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct WitnessEntities<T: Default> {
    pub elements: [T; WITNESS_ENTITIES_SIZE],
}

pub(crate) const SHIFTED_WITNESS_ENTITIES_SIZE: usize = 5;

#[derive(Default, Clone)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct ShiftedWitnessEntities<T: Default> {
    pub elements: [T; SHIFTED_WITNESS_ENTITIES_SIZE],
}

impl<T: Default> IntoIterator for ShiftedWitnessEntities<T> {
    type Item = T;
    type IntoIter = core::array::IntoIter<T, SHIFTED_WITNESS_ENTITIES_SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<T: Default> WitnessEntities<T> {
    /// column 0
    const W_L: usize = 0;
    /// column 1
    const W_R: usize = 1;
    /// column 2
    const W_O: usize = 2;
    /// column 3 (computed by prover)
    const W_4: usize = 3;
    /// column 4 (computed by prover)
    const Z_PERM: usize = 4;
    /// column 5 (computed by prover);
    pub(crate) const LOOKUP_INVERSES: usize = 5;
    /// column 6
    pub(crate) const LOOKUP_READ_COUNTS: usize = 6;
    /// column 7
    pub(crate) const LOOKUP_READ_TAGS: usize = 7;

    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        self.elements.iter()
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.elements.iter_mut()
    }

    pub(crate) fn to_be_shifted(&self) -> &[T] {
        &self.elements[Self::W_L..=Self::Z_PERM]
    }

    pub(crate) fn w_l(&self) -> &T {
        &self.elements[Self::W_L]
    }

    pub(crate) fn w_r(&self) -> &T {
        &self.elements[Self::W_R]
    }

    pub(crate) fn w_o(&self) -> &T {
        &self.elements[Self::W_O]
    }

    pub(crate) fn w_4(&self) -> &T {
        &self.elements[Self::W_4]
    }

    pub(crate) fn z_perm(&self) -> &T {
        &self.elements[Self::Z_PERM]
    }

    pub(crate) fn lookup_inverses(&self) -> &T {
        &self.elements[Self::LOOKUP_INVERSES]
    }

    pub(crate) fn lookup_read_counts(&self) -> &T {
        &self.elements[Self::LOOKUP_READ_COUNTS]
    }

    pub(crate) fn lookup_read_tags(&self) -> &T {
        &self.elements[Self::LOOKUP_READ_TAGS]
    }

    pub(crate) fn w_l_mut(&mut self) -> &mut T {
        &mut self.elements[Self::W_L]
    }

    pub(crate) fn w_r_mut(&mut self) -> &mut T {
        &mut self.elements[Self::W_R]
    }

    pub(crate) fn w_o_mut(&mut self) -> &mut T {
        &mut self.elements[Self::W_O]
    }

    pub(crate) fn w_4_mut(&mut self) -> &mut T {
        &mut self.elements[Self::W_4]
    }

    pub(crate) fn z_perm_mut(&mut self) -> &mut T {
        &mut self.elements[Self::Z_PERM]
    }

    pub(crate) fn lookup_inverses_mut(&mut self) -> &mut T {
        &mut self.elements[Self::LOOKUP_INVERSES]
    }

    pub(crate) fn lookup_read_counts_mut(&mut self) -> &mut T {
        &mut self.elements[Self::LOOKUP_READ_COUNTS]
    }

    pub(crate) fn lookup_read_tags_mut(&mut self) -> &mut T {
        &mut self.elements[Self::LOOKUP_READ_TAGS]
    }
}

impl<T: Default> ShiftedWitnessEntities<T> {
    /// column 0
    const W_L: usize = 0;
    /// column 1
    const W_R: usize = 1;
    /// column 2
    const W_O: usize = 2;
    /// column 3
    const W_4: usize = 3;
    /// column 4
    const Z_PERM: usize = 4;

    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        self.elements.iter()
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.elements.iter_mut()
    }

    pub(crate) fn w_l(&self) -> &T {
        &self.elements[Self::W_L]
    }

    pub(crate) fn w_r(&self) -> &T {
        &self.elements[Self::W_R]
    }

    pub(crate) fn w_o(&self) -> &T {
        &self.elements[Self::W_O]
    }

    pub(crate) fn w_4(&self) -> &T {
        &self.elements[Self::W_4]
    }

    pub(crate) fn z_perm(&self) -> &T {
        &self.elements[Self::Z_PERM]
    }
}

/// The number of elements in the precomputed entities array
pub(crate) const PRECOMPUTED_ENTITIES_SIZE: usize = 27;

#[derive(Default, Clone)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct PrecomputedEntities<T: Default> {
    pub elements: [T; PRECOMPUTED_ENTITIES_SIZE],
}

impl<T: Default> PrecomputedEntities<T> {
    /// column 0
    pub(crate) const Q_M: usize = 0;
    /// column 1
    pub(crate) const Q_C: usize = 1;
    /// column 2
    pub(crate) const Q_L: usize = 2;
    /// column 3
    pub(crate) const Q_R: usize = 3;
    /// column 4
    pub(crate) const Q_O: usize = 4;
    /// column 5
    pub(crate) const Q_4: usize = 5;
    /// column 6
    pub(crate) const Q_LOOKUP: usize = 6;
    /// column 7
    pub(crate) const Q_ARITH: usize = 7;
    /// column 8
    pub(crate) const Q_DELTA_RANGE: usize = 8;
    /// column 9
    pub(crate) const Q_ELLIPTIC: usize = 9;
    /// column 10
    pub(crate) const Q_AUX: usize = 10;
    /// column 11
    pub(crate) const Q_POSEIDON2_EXTERNAL: usize = 11;
    /// column 12
    pub(crate) const Q_POSEIDON2_INTERNAL: usize = 12;
    /// column 13
    const SIGMA_1: usize = 13;
    /// column 14
    const SIGMA_2: usize = 14;
    /// column 15
    const SIGMA_3: usize = 15;
    /// column 16
    const SIGMA_4: usize = 16;
    /// column 17
    const ID_1: usize = 17;
    /// column 18
    const ID_2: usize = 18;
    /// column 19
    const ID_3: usize = 19;
    /// column 20
    const ID_4: usize = 20;
    /// column 21
    const TABLE_1: usize = 21;
    /// column 22
    const TABLE_2: usize = 22;
    /// column 23
    const TABLE_3: usize = 23;
    /// column 24
    const TABLE_4: usize = 24;
    /// column 25
    const LAGRANGE_FIRST: usize = 25;
    /// column 26
    const LAGRANGE_LAST: usize = 26;

    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        self.elements.iter()
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.elements.iter_mut()
    }

    pub(crate) fn q_m(&self) -> &T {
        &self.elements[Self::Q_M]
    }

    pub(crate) fn q_c(&self) -> &T {
        &self.elements[Self::Q_C]
    }

    pub(crate) fn q_l(&self) -> &T {
        &self.elements[Self::Q_L]
    }

    pub(crate) fn q_r(&self) -> &T {
        &self.elements[Self::Q_R]
    }

    pub(crate) fn q_o(&self) -> &T {
        &self.elements[Self::Q_O]
    }

    pub(crate) fn q_4(&self) -> &T {
        &self.elements[Self::Q_4]
    }

    pub(crate) fn q_arith(&self) -> &T {
        &self.elements[Self::Q_ARITH]
    }

    pub(crate) fn q_delta_range(&self) -> &T {
        &self.elements[Self::Q_DELTA_RANGE]
    }

    pub(crate) fn q_elliptic(&self) -> &T {
        &self.elements[Self::Q_ELLIPTIC]
    }

    pub(crate) fn q_aux(&self) -> &T {
        &self.elements[Self::Q_AUX]
    }

    pub(crate) fn q_lookup(&self) -> &T {
        &self.elements[Self::Q_LOOKUP]
    }

    pub(crate) fn q_poseidon2_external(&self) -> &T {
        &self.elements[Self::Q_POSEIDON2_EXTERNAL]
    }

    pub(crate) fn q_poseidon2_internal(&self) -> &T {
        &self.elements[Self::Q_POSEIDON2_INTERNAL]
    }

    pub(crate) fn sigma_1(&self) -> &T {
        &self.elements[Self::SIGMA_1]
    }

    pub(crate) fn sigma_2(&self) -> &T {
        &self.elements[Self::SIGMA_2]
    }

    pub(crate) fn sigma_3(&self) -> &T {
        &self.elements[Self::SIGMA_3]
    }

    pub(crate) fn sigma_4(&self) -> &T {
        &self.elements[Self::SIGMA_4]
    }

    pub(crate) fn id_1(&self) -> &T {
        &self.elements[Self::ID_1]
    }

    pub(crate) fn id_2(&self) -> &T {
        &self.elements[Self::ID_2]
    }

    pub(crate) fn id_3(&self) -> &T {
        &self.elements[Self::ID_3]
    }

    pub(crate) fn id_4(&self) -> &T {
        &self.elements[Self::ID_4]
    }

    pub(crate) fn table_1(&self) -> &T {
        &self.elements[Self::TABLE_1]
    }

    pub(crate) fn table_2(&self) -> &T {
        &self.elements[Self::TABLE_2]
    }

    pub(crate) fn table_3(&self) -> &T {
        &self.elements[Self::TABLE_3]
    }

    pub(crate) fn table_4(&self) -> &T {
        &self.elements[Self::TABLE_4]
    }

    pub(crate) fn lagrange_first(&self) -> &T {
        &self.elements[Self::LAGRANGE_FIRST]
    }

    pub(crate) fn lagrange_last(&self) -> &T {
        &self.elements[Self::LAGRANGE_LAST]
    }
}
