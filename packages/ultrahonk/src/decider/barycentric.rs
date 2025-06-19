use alloc::vec::Vec;
use ark_ff::PrimeField;

pub struct Barycentric {}

impl Barycentric {
    // Methods for computing arrays of precomputable data used for barycentric extension and evaluation

    // build big_domain, currently the set of x_i in {domain_start, ..., big_domain_end - 1 }
    pub fn construct_big_domain<F: PrimeField>(domain_size: usize, num_evals: usize) -> Vec<F> {
        let big_domain_size = core::cmp::max(domain_size, num_evals);
        let mut res = Vec::with_capacity(big_domain_size);
        for i in 0..big_domain_size {
            res.push(F::from(i as u64));
        }
        res
    }

    // build set of lagrange_denominators d_i = \prod_{j!=i} x_i - x_j
    pub fn construct_lagrange_denominators<F: PrimeField>(
        domain_size: usize,
        big_domain: &[F],
    ) -> Vec<F> {
        let mut res = Vec::with_capacity(domain_size);

        for i in 0..domain_size {
            let mut r = F::one();
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
