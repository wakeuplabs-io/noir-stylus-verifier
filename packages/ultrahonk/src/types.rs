pub use crate::polynomials::polynomial_types::{PrecomputedEntities, PRECOMPUTED_ENTITIES_SIZE};
use crate::serialize::Serialize;
use crate::HonkProofResult;
use ark_bn254::{g1::Config as G1Config, g2::Config as G2Config, Fq, Fq2, Fr};
use ark_ec::short_weierstrass::Affine;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HonkProof {
    proof: Vec<ScalarField>,
}

impl HonkProof {
    pub(crate) fn new(proof: Vec<ScalarField>) -> Self {
        Self { proof }
    }

    pub fn inner(self) -> Vec<ScalarField> {
        self.proof
    }

    pub fn to_buffer(&self) -> Vec<u8> {
        Serialize::to_buffer(&self.proof, false)
    }

    pub fn from_buffer(buf: &[u8]) -> HonkProofResult<Self> {
        let res = Serialize::from_buffer(buf, false)?;
        Ok(Self::new(res))
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
#[derive(Default)]
pub(crate) struct AllEntities<T: Default> {
    pub(crate) witness: WitnessEntities<T>,
    pub(crate) precomputed: PrecomputedEntities<T>,
    pub(crate) shifted_witness: ShiftedWitnessEntities<T>,
}

impl<T: Default> AllEntities<T> {
    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.precomputed
            .iter_mut()
            .chain(self.witness.iter_mut())
            .chain(self.shifted_witness.iter_mut())
    }
}

const WITNESS_ENTITIES_SIZE: usize = 8;
#[derive(Default, Clone)]
pub struct WitnessEntities<T: Default> {
    pub(crate) elements: [T; WITNESS_ENTITIES_SIZE],
}

const SHIFTED_WITNESS_ENTITIES_SIZE: usize = 5;
#[derive(Default, Clone)]
pub struct ShiftedWitnessEntities<T: Default> {
    pub(crate) elements: [T; SHIFTED_WITNESS_ENTITIES_SIZE],
}

impl<T: Default> IntoIterator for WitnessEntities<T> {
    type Item = T;
    type IntoIter = std::array::IntoIter<T, WITNESS_ENTITIES_SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<T: Default> WitnessEntities<Vec<T>> {
    pub fn new() -> Self {
        Self {
            elements: std::array::from_fn(|_| Vec::new()),
        }
    }

    pub fn add(&mut self, witness_entity: WitnessEntities<T>) {
        for (src, dst) in witness_entity.into_iter().zip(self.iter_mut()) {
            dst.push(src);
        }
    }
}

impl<T: Default> IntoIterator for ShiftedWitnessEntities<T> {
    type Item = T;
    type IntoIter = std::array::IntoIter<T, SHIFTED_WITNESS_ENTITIES_SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<T: Default> ShiftedWitnessEntities<Vec<T>> {
    pub fn new() -> Self {
        Self {
            elements: std::array::from_fn(|_| Vec::new()),
        }
    }

    pub fn add(&mut self, shifted_witness_entities: ShiftedWitnessEntities<T>) {
        for (src, dst) in shifted_witness_entities.into_iter().zip(self.iter_mut()) {
            dst.push(src);
        }
    }
}

impl<T: Default> IntoIterator for AllEntities<T> {
    type Item = T;
    type IntoIter = std::iter::Chain<
        std::iter::Chain<
            std::array::IntoIter<T, PRECOMPUTED_ENTITIES_SIZE>,
            std::array::IntoIter<T, WITNESS_ENTITIES_SIZE>,
        >,
        std::array::IntoIter<T, SHIFTED_WITNESS_ENTITIES_SIZE>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.precomputed
            .into_iter()
            .chain(self.witness)
            .chain(self.shifted_witness)
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

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.elements.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.elements.iter_mut()
    }

    pub fn to_be_shifted(&self) -> &[T] {
        &self.elements[Self::W_L..=Self::Z_PERM]
    }

    pub fn to_be_shifted_mut(&mut self) -> &mut [T] {
        &mut self.elements[Self::W_L..=Self::Z_PERM]
    }

    pub fn w_l(&self) -> &T {
        &self.elements[Self::W_L]
    }

    pub fn w_r(&self) -> &T {
        &self.elements[Self::W_R]
    }

    pub fn w_o(&self) -> &T {
        &self.elements[Self::W_O]
    }

    pub fn w_4(&self) -> &T {
        &self.elements[Self::W_4]
    }

    pub fn z_perm(&self) -> &T {
        &self.elements[Self::Z_PERM]
    }

    pub fn lookup_inverses(&self) -> &T {
        &self.elements[Self::LOOKUP_INVERSES]
    }

    pub fn lookup_read_counts(&self) -> &T {
        &self.elements[Self::LOOKUP_READ_COUNTS]
    }

    pub fn lookup_read_tags(&self) -> &T {
        &self.elements[Self::LOOKUP_READ_TAGS]
    }

    pub fn w_l_mut(&mut self) -> &mut T {
        &mut self.elements[Self::W_L]
    }

    pub fn w_r_mut(&mut self) -> &mut T {
        &mut self.elements[Self::W_R]
    }

    pub fn w_o_mut(&mut self) -> &mut T {
        &mut self.elements[Self::W_O]
    }

    pub fn w_4_mut(&mut self) -> &mut T {
        &mut self.elements[Self::W_4]
    }

    pub fn z_perm_mut(&mut self) -> &mut T {
        &mut self.elements[Self::Z_PERM]
    }

    pub fn lookup_inverses_mut(&mut self) -> &mut T {
        &mut self.elements[Self::LOOKUP_INVERSES]
    }

    pub fn lookup_read_counts_mut(&mut self) -> &mut T {
        &mut self.elements[Self::LOOKUP_READ_COUNTS]
    }

    pub fn lookup_read_tags_mut(&mut self) -> &mut T {
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

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.elements.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.elements.iter_mut()
    }

    pub fn w_l(&self) -> &T {
        &self.elements[Self::W_L]
    }

    pub fn w_r(&self) -> &T {
        &self.elements[Self::W_R]
    }

    pub fn w_o(&self) -> &T {
        &self.elements[Self::W_O]
    }

    pub fn w_4(&self) -> &T {
        &self.elements[Self::W_4]
    }

    pub fn z_perm(&self) -> &T {
        &self.elements[Self::Z_PERM]
    }

    pub fn w_l_mut(&mut self) -> &mut T {
        &mut self.elements[Self::W_L]
    }

    pub fn w_r_mut(&mut self) -> &mut T {
        &mut self.elements[Self::W_R]
    }

    pub fn w_o_mut(&mut self) -> &mut T {
        &mut self.elements[Self::W_O]
    }

    pub fn w_4_mut(&mut self) -> &mut T {
        &mut self.elements[Self::W_4]
    }

    pub fn z_perm_mut(&mut self) -> &mut T {
        &mut self.elements[Self::Z_PERM]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ZeroKnowledge {
    No,
    Yes,
}

impl From<bool> for ZeroKnowledge {
    fn from(value: bool) -> Self {
        if value {
            ZeroKnowledge::Yes
        } else {
            ZeroKnowledge::No
        }
    }
}
