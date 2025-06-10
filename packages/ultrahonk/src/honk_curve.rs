use ark_bn254::{Fr};
use ark_ff::{Field};

use crate::backends::G1ArithmeticBackend;

// TODO: comments explaining this
pub const NUM_BASEFIELD_ELEMENTS: usize = 2;
pub const NUM_SCALARFIELD_ELEMENTS: usize = 1;
pub const SUBGROUP_SIZE: usize = 256;
pub const LIBRA_UNIVARIATES_LENGTH: usize = 9;


pub trait HonkCurve: G1ArithmeticBackend {
    fn get_curve_b() -> Fr;

    fn get_subgroup_generator() -> Fr;

    fn get_subgroup_generator_inverse() -> Fr {
        Self::get_subgroup_generator().inverse().unwrap()
    }
}
