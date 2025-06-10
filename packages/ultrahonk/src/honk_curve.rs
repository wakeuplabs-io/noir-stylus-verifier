use ark_bn254::{Fr};
use ark_ec::{pairing::Pairing, CurveGroup};
use ark_ff::{BigInt, Field};
use std::str::FromStr;
use crate::types::{ScalarField};

// TODO: comments explaining this

pub const NUM_BASEFIELD_ELEMENTS: usize = 2;
pub const NUM_SCALARFIELD_ELEMENTS: usize = 1;
pub const SUBGROUP_SIZE: usize = 256;
pub const LIBRA_UNIVARIATES_LENGTH: usize = 9;

pub const NUM_LIMB_BITS: u32 = 68;
pub const TOTAL_BITS: u32 = 254;

// Des describes the PrimeField used for the Transcript
pub trait HonkCurve: Pairing {
    type CycleGroup: CurveGroup<BaseField = Self::ScalarField>;

    fn get_curve_b() -> Fr;

    fn get_subgroup_generator() -> Fr;

    fn get_subgroup_generator_inverse() -> Fr {
        Self::get_subgroup_generator().inverse().unwrap()
    }
}

// TODO: Move implementation to tests?
impl HonkCurve for ark_bn254::Bn254 {
    type CycleGroup = ark_grumpkin::Projective;

    fn get_curve_b() -> Self::ScalarField {
        // We are getting grumpkin::b, which is -17
        -ScalarField::from(17)
    }

    fn get_subgroup_generator() -> Self::ScalarField {
        let val = ark_bn254::Fr::from(BigInt::new([
            14453002906517207670,
            7023718024139043376,
            17331575720852783024,
            554159777355432964,
        ]));
        debug_assert_eq!(
            val,
            ark_bn254::Fr::from_str(
                "3478517300119284901893091970156912948790432420133812234316178878452092729974",
            )
            .unwrap()
        );

        val
    }

    fn get_subgroup_generator_inverse() -> Self::ScalarField {
        let val = ark_bn254::Fr::from(BigInt::new([
            7578525993492149718,
            11911168646041470090,
            7238721496332547558,
            2327185798872627923,
        ]));
        debug_assert_eq!(val, Self::get_subgroup_generator().inverse().unwrap());
        val
    }
}


