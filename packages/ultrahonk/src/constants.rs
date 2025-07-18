use crate::serialize::BytesDeserializable;
use crate::types::{G2Affine, ScalarField};

/// The number of bytes to represent field elements of the base or scalar fields
/// for the G1 curve group, as well as the base field which is extended for the
/// G2 curve group
pub const NUM_BYTES_FELT: usize = 32;

/// The number of u64s it takes to represent a field element
pub const NUM_U64S_FELT: usize = 4;

/// The number of bytes it takes to represent a u64
pub const NUM_BYTES_U64: usize = 8;

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

// For ZK Flavors: the number of the commitments required by Libra and SmallSubgroupIPA.
pub const NUM_LIBRA_COMMITMENTS: usize = 3;

/// The maximum partial relation length.
pub const MAX_PARTIAL_RELATION_LENGTH: usize = 7;
pub const BATCHED_RELATION_PARTIAL_LENGTH: usize = MAX_PARTIAL_RELATION_LENGTH + 1;
pub const BATCHED_RELATION_PARTIAL_LENGTH_ZK: usize = BATCHED_RELATION_PARTIAL_LENGTH + 1;

// We are getting grumpkin::b, which is -17
pub(crate) fn get_honk_curve_b() -> ScalarField {
    -ScalarField::from(17)
}

pub(crate) fn get_crs_g2() -> G2Affine {
    G2Affine::deserialize_from_bytes(
        // hex::decode("260e01b251f6f1c7e7ff4e580791dee8ea51d87a358e038b4efe30fac09383c10118c4d5b837bcc2bc89b5b398b5974e9f5944073b32078b7e231fec938883b004fc6369f7110fe3d25156c1bb9a72859cf2a04641f99ba4ee413c80da6a5fe422febda3c0c0632a56475b4214e5615e11e6dd3f96e6cea2854a87d4dacc5e55").unwrap().as_slice()
        &[
            38, 14, 1, 178, 81, 246, 241, 199, 231, 255, 78, 88, 7, 145, 222, 232, 234, 81, 216,
            122, 53, 142, 3, 139, 78, 254, 48, 250, 192, 147, 131, 193, 1, 24, 196, 213, 184, 55,
            188, 194, 188, 137, 181, 179, 152, 181, 151, 78, 159, 89, 68, 7, 59, 50, 7, 139, 126,
            35, 31, 236, 147, 136, 131, 176, 4, 252, 99, 105, 247, 17, 15, 227, 210, 81, 86, 193,
            187, 154, 114, 133, 156, 242, 160, 70, 65, 249, 155, 164, 238, 65, 60, 128, 218, 106,
            95, 228, 34, 254, 189, 163, 192, 192, 99, 42, 86, 71, 91, 66, 20, 229, 97, 94, 17, 230,
            221, 63, 150, 230, 206, 162, 133, 74, 135, 212, 218, 204, 94, 85,
        ],
    )
    .unwrap()
    .0
}

pub(crate) fn get_subgroup_generator() -> ScalarField {
    ScalarField::deserialize_from_bytes(
        // hex::decode("07b0c561a6148404f086204a9f36ffb0617942546750f230c893619174a57a76")
        &[
            7, 176, 197, 97, 166, 20, 132, 4, 240, 134, 32, 74, 159, 54, 255, 176, 97, 121, 66, 84,
            103, 80, 242, 48, 200, 147, 97, 145, 116, 165, 122, 118,
        ],
    )
    .unwrap()
    .0
}

pub(crate) fn get_subgroup_generator_inverse() -> ScalarField {
    ScalarField::deserialize_from_bytes(
        // hex::decode("204bd3277422fad364751ad938e2b5e6a54cf8c68712848a692c553d0329f5d6")
        &[
            32, 75, 211, 39, 116, 34, 250, 211, 100, 117, 26, 217, 56, 226, 181, 230, 165, 76, 248,
            198, 135, 18, 132, 138, 105, 44, 85, 61, 3, 41, 245, 214,
        ],
    )
    .unwrap()
    .0
}
