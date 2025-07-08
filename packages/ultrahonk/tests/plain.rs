#![macro_use]
extern crate alloc;

use alloc::vec::Vec;
use ark_bn254::Bn254;
use ark_ec::{pairing::Pairing, AffineRepr, CurveGroup};
use ark_ff::One;
use sha3::{Digest, Keccak256};
use ultrahonk::{
    backends::{G1ArithmeticBackend, G1ArithmeticError, HashBackend},
    constants::HASH_OUTPUT_SIZE,
    keys::verification_key::VerifyingKey,
    serialize::BytesDeserializable,
    types::{G1Affine, G2Affine, HonkProof, ScalarField},
    verifier::UltraHonk,
};

pub struct ArkKeccak256;

impl HashBackend for ArkKeccak256 {
    fn hash(buffer: &[u8]) -> [u8; HASH_OUTPUT_SIZE] {
        // Losing 2 bits of this is not an issue -> we can just reduce mod p
        let mut hasher = Keccak256::default();
        hasher.update(buffer);
        let hash_result = hasher.finalize();
        hash_result.try_into().unwrap()
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
    ($(($name:ident, $path:expr)),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                let path = std::path::Path::new($path);
                let proof_file = format!("{}/kat/proof", path.display());
                let vk_file = format!("{}/kat/vk", path.display());
                let public_inputs_file = format!("{}/kat/public_inputs", path.display());

                // parse proof file
                let proof_u8 = std::fs::read(proof_file).unwrap();
                let proof = HonkProof::from_buffer(&proof_u8).unwrap();

                // parse public_inputs file
                let public_inputs_u8 = std::fs::read(public_inputs_file).unwrap();
                let public_inputs = Vec::<ScalarField>::deserialize_from_bytes(&public_inputs_u8).unwrap();

                // parse verification key file
                let vk_u8 = std::fs::read(vk_file).unwrap();
                let vk = VerifyingKey::from_buffer(&vk_u8).unwrap();

                let is_valid =
                    UltraHonk::verify::<ArkKeccak256, ArkHonkCurve>(proof, &public_inputs, &vk).unwrap();

                assert!(is_valid);
            }
        )*
    };
}


// Run this to generate the tests:
// echo "generate_tests!(" && for d in ../../test_vectors/*; do [ -d "$d" ] && name=$(basename "$d" | tr -c '[:alnum:]' '_' | sed 's/__*/_/g') && echo "    ($name, \"$d\")," ; done && echo ");"
generate_tests!(
    (add3u64_, "../../test_vectors/add3u64"),
    (addition_multiplication_, "../../test_vectors/addition_multiplication"),
    (approx_sigmoid_, "../../test_vectors/approx_sigmoid"),
    (assert_, "../../test_vectors/assert"),
    (bb_sha256_compression_, "../../test_vectors/bb_sha256_compression"),
    (get_bytes_, "../../test_vectors/get_bytes"),
    (if_then_, "../../test_vectors/if_then"),
    (negative_, "../../test_vectors/negative"),
    (poseidon_, "../../test_vectors/poseidon"),
    (poseidon_assert_, "../../test_vectors/poseidon_assert"),
    (poseidon_input2_, "../../test_vectors/poseidon_input2"),
    (poseidon_stdlib_, "../../test_vectors/poseidon_stdlib"),
    (poseidon2_, "../../test_vectors/poseidon2"),
    (quantized_, "../../test_vectors/quantized"),
    (random_access_, "../../test_vectors/random_access"),
    (slice_, "../../test_vectors/slice"),
    (to_radix32_, "../../test_vectors/to_radix32"),
    (unconstrained_fn_, "../../test_vectors/unconstrained_fn"),
    (unconstrained_fn_field_, "../../test_vectors/unconstrained_fn_field"),
    (write_access_, "../../test_vectors/write_access"),
);
