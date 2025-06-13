/// The last byte of the `ecAdd` precompile address, 0x06
pub const EC_ADD_ADDRESS_LAST_BYTE: u8 = 6;

/// The last byte of the `ecMul` precompile address, 0x07
pub const EC_MUL_ADDRESS_LAST_BYTE: u8 = 7;

/// The last byte of the `ecPairing` precompile address, 0x08
pub const EC_PAIRING_ADDRESS_LAST_BYTE: u8 = 8;

/// The index of the last byte of the `ecPairing` precompile result,
/// which is a boolean indicating whether the pairing check succeeded
pub const PAIRING_CHECK_RESULT_LAST_BYTE_INDEX: usize = 31;

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
