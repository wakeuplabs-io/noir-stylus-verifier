use crate::types::ScalarField;
use alloc::vec::Vec;
use ark_ff::One;

pub(crate) struct Barycentric {}

/// Methods for computing arrays of precomputable data used for barycentric extension and evaluation
impl Barycentric {
    /// Build big_domain, currently the set of x_i in {domain_start, ..., big_domain_end - 1 }
    pub(crate) fn construct_big_domain(domain_size: usize, num_evals: usize) -> Vec<ScalarField> {
        let big_domain_size = core::cmp::max(domain_size, num_evals);
        let mut res = Vec::with_capacity(big_domain_size);
        for i in 0..big_domain_size {
            res.push(ScalarField::from(i as u64));
        }
        res
    }

    /// Build set of lagrange_denominators d_i = \prod_{j!=i} x_i - x_j
    pub(crate) fn construct_lagrange_denominators(
        domain_size: usize,
        big_domain: &[ScalarField],
    ) -> Vec<ScalarField> {
        let mut res = Vec::with_capacity(domain_size);

        for i in 0..domain_size {
            let mut r = ScalarField::one();
            for j in 0..domain_size {
                if j != i {
                    r *= big_domain[i] - big_domain[j];
                }
            }
            res.push(r);
        }
        res
    }
}
