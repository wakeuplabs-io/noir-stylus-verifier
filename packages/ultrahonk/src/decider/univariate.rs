use crate::decider::barycentric::Barycentric;
use ark_ff::{PrimeField};

#[derive(Clone, Debug)]
pub struct Univariate<F, const SIZE: usize> {
    pub evaluations: [F; SIZE],
}

impl<F: PrimeField, const SIZE: usize> Univariate<F, SIZE> {
    pub(crate) fn evaluate(&self, u: F) -> F {
        if u == F::zero() {
            return self.evaluations[0];
        }

        let mut full_numerator_value = F::one();
        for i in 0..SIZE {
            full_numerator_value *= u - F::from(i as u64);
        }

        let big_domain = Barycentric::construct_big_domain(self.evaluations.len(), SIZE);
        let lagrange_denominators = Barycentric::construct_lagrange_denominators(SIZE, &big_domain);

        let mut denominator_inverses = [F::zero(); SIZE];
        for i in 0..SIZE {
            let mut inv = lagrange_denominators[i];

            inv *= u - big_domain[i];
            inv = F::one() / inv;
            denominator_inverses[i] = inv;
        }

        let mut result = F::zero();
        // Compute each term v_j / (d_j*(x-x_j)) of the sum
        for (i, &inverse) in denominator_inverses.iter().enumerate() {
            let mut term = self.evaluations[i];
            term *= inverse;
            result += term;
        }

        // Scale the sum by the value of B(x)
        result *= full_numerator_value;
        result
    }

}
