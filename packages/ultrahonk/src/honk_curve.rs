use ark_ff::Field;
use crate::{backends::G1ArithmeticBackend, types::ScalarField};

/// The number of base field elements in the ultrahonk::HonkCurve representation.
///
/// This is the number of elements required to represent the G1 curve.
pub const NUM_BASEFIELD_ELEMENTS: usize = 2;

/// The number of scalar field elements in the ultrahonk::HonkCurve representation.
///
/// This is the number of elements required to represent the scalar field.
pub const NUM_SCALARFIELD_ELEMENTS: usize = 1;

/// The size of the subgroup in the ultrahonk::HonkCurve representation.
///
/// This is the order of the subgroup of G1.
pub const SUBGROUP_SIZE: usize = 256;

/// The number of univariate polynomials in the ultrahonk::HonkCurve representation used by Libra.
///
/// This is the number of polynomials required to encode the Libra VM's state.
pub const LIBRA_UNIVARIATES_LENGTH: usize = 9;

pub trait HonkCurve: G1ArithmeticBackend {
    fn get_curve_b() -> ScalarField;

    fn get_subgroup_generator() -> ScalarField;

    fn get_subgroup_generator_inverse() -> ScalarField {
        Self::get_subgroup_generator().inverse().unwrap()
    }
}
