use crate::{backends::G1ArithmeticBackend, types::ScalarField};
use ark_ff::{BigInt, Field};
use core::str::FromStr;

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

// TODO: improve this, make constants instead
pub trait HonkCurve: G1ArithmeticBackend {
    fn get_curve_b() -> ScalarField {
        // We are getting grumpkin::b, which is -17
        -ScalarField::from(17)
    }

    fn get_subgroup_generator() -> ScalarField {
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

    fn get_subgroup_generator_inverse() -> ScalarField {
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
