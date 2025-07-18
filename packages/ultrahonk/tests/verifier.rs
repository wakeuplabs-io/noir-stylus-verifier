#![macro_use]
extern crate alloc;

use alloc::vec::Vec;
use ark_bn254::Bn254;
use ark_ec::{pairing::Pairing, AffineRepr, CurveGroup};
use ark_ff::One;
use paste::paste;
use sha3::{Digest, Keccak256};
use std::env;
use ultrahonk::{
    backends::{G1ArithmeticBackend, G1ArithmeticError, HashBackend},
    constants::HASH_OUTPUT_SIZE,
    serialize::BytesDeserializable,
    types::{G1Affine, G2Affine, HonkProof, ScalarField, VerifyingKey},
    verifier::UltraHonk,
};

pub struct ArkKeccak256;

impl HashBackend for ArkKeccak256 {
    fn hash(buffer: &[u8]) -> [u8; HASH_OUTPUT_SIZE] {
        // Losing 2 bits of this is not an issue -> we can just reduce mod p
        let mut hasher = Keccak256::default();
        hasher.update(buffer);
        let hash_result = hasher.finalize();
        hash_result.into()
    }
}

pub struct ArkHonkCurve;

impl G1ArithmeticBackend for ArkHonkCurve {
    /// Add two points in G1
    fn ec_add(a: G1Affine, b: G1Affine) -> Result<G1Affine, G1ArithmeticError> {
        Ok((a + b).into_affine())
    }

    /// Multiply a G1 point by a scalar in its scalar field
    fn ec_scalar_mul(a: ScalarField, b: G1Affine) -> Result<G1Affine, G1ArithmeticError> {
        let mut b_group = b.into_group();
        b_group *= a;
        Ok(b_group.into_affine())
    }

    /// Check the pairing identity e(a_1, b_1) == e(a_2, b_2)
    fn ec_pairing_check(
        p0: G1Affine,
        p1: G1Affine,
        g2_x: G2Affine,
        g2_gen: G2Affine,
    ) -> Result<bool, G1ArithmeticError> {
        let p = [g2_gen, g2_x];
        let g1_prepared = [
            <Bn254 as Pairing>::G1Prepared::from(p0),
            <Bn254 as Pairing>::G1Prepared::from(p1),
        ];
        Ok(<Bn254 as Pairing>::multi_pairing(g1_prepared, p).0
            == <Bn254 as Pairing>::TargetField::one())
    }

    fn msm(scalars: &[ScalarField], points: &[G1Affine]) -> Result<G1Affine, G1ArithmeticError> {
        if scalars.len() != points.len() {
            return Err(G1ArithmeticError);
        }

        scalars
            .iter()
            .zip(points.iter())
            .try_fold(G1Affine::identity(), |acc, (scalar, point)| {
                let scaled_point = Self::ec_scalar_mul(*scalar, *point)?;
                Self::ec_add(acc, scaled_point)
            })
    }
}

macro_rules! generate_tests {
    ($($name:ident),* $(,)?) => {
        $(
            paste! {
                #[test]
                fn [<$name _test>]() {
                    // build path to test vector data
                    let workspace_path = std::path::Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
                        .ancestors()
                        .nth(2) // Go up 2 levels: ultrahonk/ -> packages/ -> workspace root
                        .unwrap()
                        .to_path_buf();
                    let test_vector_path = workspace_path.join("test_vectors").join(stringify!($name));

                    // parse proof file
                    let proof = HonkProof::deserialize_from_bytes(&std::fs::read(test_vector_path.join("kat/proof")).unwrap()).unwrap().0;
                    let public_inputs = Vec::<ScalarField>::deserialize_from_bytes(&std::fs::read(test_vector_path.join("kat/public_inputs")).unwrap()).unwrap().0;
                    let vk = VerifyingKey::deserialize_from_bytes(&std::fs::read(test_vector_path.join("kat/vk")).unwrap()).unwrap().0;

                    // verify proofs
                    assert!(UltraHonk::verify::<ArkKeccak256, ArkHonkCurve>(proof, &public_inputs, &vk, false).unwrap());
                }
            }

            paste! {
                #[test]
                fn [<zk_$name _test>]() {
                    // build path to test vector data
                    let workspace_path = std::path::Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
                        .ancestors()
                        .nth(2) // Go up 2 levels: ultrahonk/ -> packages/ -> workspace root
                        .unwrap()
                        .to_path_buf();
                    let test_vector_path = workspace_path.join("test_vectors").join(stringify!($name));

                    // parse proof file
                    let zk_proof = HonkProof::deserialize_from_bytes(&std::fs::read(test_vector_path.join("kat/zk-proof")).unwrap()).unwrap().0;
                    let public_inputs = Vec::<ScalarField>::deserialize_from_bytes(&std::fs::read(test_vector_path.join("kat/public_inputs")).unwrap()).unwrap().0;
                    let vk = VerifyingKey::deserialize_from_bytes(&std::fs::read(test_vector_path.join("kat/vk")).unwrap()).unwrap().0;

                    // verify proofs
                    assert!(UltraHonk::verify::<ArkKeccak256, ArkHonkCurve>(zk_proof, &public_inputs, &vk, true).unwrap());
                }
            }
        )*
    };
}

// Run this to generate the tests:
// echo "generate_tests!(" && for d in ./test_vectors/*; do [ -d "$d" ] && name=$(basename "$d" | sed 's/__*/_/g') && echo "    $name," ; done && echo ");"
generate_tests!(
    add3u64,
    addition_multiplication,
    approx_sigmoid,
    assert,
    bb_sha256_compression,
    get_bytes,
    if_then,
    negative,
    poseidon,
    poseidon_assert,
    poseidon_input2,
    poseidon_stdlib,
    poseidon2,
    quantized,
    random_access,
    slice,
    to_radix32,
    unconstrained_fn,
    unconstrained_fn_field,
    write_access,
);
