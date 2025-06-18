use crate::serde_compat;
use alloc::vec::Vec;
use ark_ff::PrimeField;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use core::ops::{AddAssign, Index, IndexMut, MulAssign, SubAssign};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// The number of last rows in ProverPolynomials that are randomized to mask
// 1) witness commitments,
// 2) multilinear evaluations of witness polynomials in Sumcheck
// 3*) multilinear evaluations of shifts of witness polynomials in Sumcheck OR univariate evaluations required in ECCVM
pub const NUM_MASKED_ROWS: u32 = 3;

#[derive(Clone, Debug, Default)]
pub struct Polynomial<F> {
    pub coefficients: Vec<F>,
}

impl<F: CanonicalSerialize> Serialize for Polynomial<F> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serde_compat::ark_se(&self.coefficients, serializer)
    }
}

impl<'a, F: CanonicalDeserialize> Deserialize<'a> for Polynomial<F> {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let coefficients: Vec<F> = serde_compat::ark_de(deserializer)?;
        Ok(Self { coefficients })
    }
}

pub struct ShiftedPoly<'a, F> {
    pub(crate) coefficients: &'a [F],
    zero: F, // TACEO TODO is there a better solution
}

impl<F: Clone> Index<usize> for ShiftedPoly<'_, F> {
    type Output = F;

    fn index(&self, index: usize) -> &Self::Output {
        if index == self.coefficients.len() {
            &self.zero
        } else {
            &self.coefficients[index]
        }
    }
}

impl<F: Clone> AsRef<[F]> for Polynomial<F> {
    fn as_ref(&self) -> &[F] {
        &self.coefficients
    }
}

impl<F: Clone> AsMut<[F]> for Polynomial<F> {
    fn as_mut(&mut self) -> &mut [F] {
        &mut self.coefficients
    }
}

impl<F: Clone> Polynomial<F> {
    pub fn new(coefficients: Vec<F>) -> Self {
        Self { coefficients }
    }

    pub fn iter(&self) -> impl Iterator<Item = &F> {
        self.coefficients.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut F> {
        self.coefficients.iter_mut()
    }

    pub fn is_empty(&self) -> bool {
        self.coefficients.is_empty()
    }

    pub fn len(&self) -> usize {
        self.coefficients.len()
    }

    pub fn resize(&mut self, size: usize, value: F) {
        self.coefficients.resize(size, value);
    }

    pub fn into_vec(self) -> Vec<F> {
        self.coefficients
    }
}

impl<F> Index<usize> for Polynomial<F> {
    type Output = F;

    fn index(&self, index: usize) -> &Self::Output {
        &self.coefficients[index]
    }
}

impl<F> IndexMut<usize> for Polynomial<F> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.coefficients[index]
    }
}

impl<F: PrimeField> AddAssign<&[F]> for Polynomial<F> {
    fn add_assign(&mut self, rhs: &[F]) {
        if rhs.len() > self.coefficients.len() {
            panic!("Polynomial too large, this should not have happened");
            // self.coefficients.resize(rhs.len(), F::zero());
        }
        for (l, r) in self.coefficients.iter_mut().zip(rhs.iter()) {
            *l += *r;
        }
    }
}

impl<F: PrimeField> SubAssign<&[F]> for Polynomial<F> {
    fn sub_assign(&mut self, rhs: &[F]) {
        if rhs.len() > self.coefficients.len() {
            panic!("Polynomial too large, this should not have happened");
            // self.coefficients.resize(rhs.len(), F::zero());
        }
        for (l, r) in self.coefficients.iter_mut().zip(rhs.iter()) {
            *l -= *r;
        }
    }
}

impl<F: PrimeField> MulAssign<F> for Polynomial<F> {
    fn mul_assign(&mut self, rhs: F) {
        for l in self.coefficients.iter_mut() {
            *l *= rhs;
        }
    }
}
