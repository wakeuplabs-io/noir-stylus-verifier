use crate::types::ScalarField;

/// The number of bytes to represent field elements of the base or scalar fields
/// for the G1 curve group, as well as the base field which is extended for the
/// G2 curve group
pub const NUM_BYTES_FELT: usize = 32;

/// The number of u64s it takes to represent a field element
pub const NUM_U64S_FELT: usize = 4;

/// The number of bytes it takes to represent a u64
pub const NUM_BYTES_U64: usize = 8;

/// The number of bits used to represent the fractional part of a real number in
/// the fixed-point representation used in the Renegade darkpool
///
/// That is, a fixed-point representation of a real number `r` is:
///     floor(r * 2^FIXED_POINT_PRECISION_BITS)
pub const FIXED_POINT_PRECISION_BITS: u64 = 63;

/// The number of bytes in a hash digest used by the transcript
pub const HASH_OUTPUT_SIZE: usize = 32;

/// The number of base field elements in the ultrahonk::HonkCurve representation.
///
/// This is the number of elements required to represent the G1 curve.
pub const NUM_BASEFIELD_ELEMENTS: usize = 2;

/// The number of scalar field elements in the ultrahonk::HonkCurve representation.
///
/// This is the number of elements required to represent the scalar field.
pub const NUM_SCALARFIELD_ELEMENTS: usize = 1;

/// The number of elements in the precomputed entities array
pub const PRECOMPUTED_ENTITIES_SIZE: usize = 27;

// We are getting grumpkin::b, which is -17. Cannot use a static because it is not a constant and avoid using Lazy to avoid size bloat.
pub fn get_honk_curve_b() -> ScalarField {
    -ScalarField::from(17)
}