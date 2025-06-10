use ark_bn254::{Fr};
use ark_ff::{Field};

// TODO: comments explaining this

pub const NUM_BASEFIELD_ELEMENTS: usize = 2;
pub const NUM_SCALARFIELD_ELEMENTS: usize = 1;
pub const SUBGROUP_SIZE: usize = 256;
pub const LIBRA_UNIVARIATES_LENGTH: usize = 9;

pub const NUM_LIMB_BITS: u32 = 68;
pub const TOTAL_BITS: u32 = 254;

pub trait HonkCurve {
    fn get_curve_b() -> Fr;

    fn get_subgroup_generator() -> Fr;

    fn get_subgroup_generator_inverse() -> Fr {
        Self::get_subgroup_generator().inverse().unwrap()
    }
}
