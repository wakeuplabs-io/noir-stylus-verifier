#![macro_use]
extern crate alloc;

use alloc::vec::Vec;
use ark_bn254::Bn254;
use ark_ec::VariableBaseMSM;
use ark_ec::{pairing::Pairing, AffineRepr, CurveGroup};
use ark_ff::One;
use sha3::{Digest, Keccak256};
use ultrahonk::{
    backends::{G1ArithmeticBackend, G1ArithmeticError, HashBackend},
    constants::HASH_OUTPUT_SIZE,
    crs::parser::CrsParser,
    keys::verification_key::{VerifyingKey, VerifyingKeyBarretenberg},
    serialize::BytesDeserializable,
    types::{G1Affine, G2Affine, ScalarField, HonkProof},
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

    /// A helper for computing multi-scalar multiplications over G1
    fn msm(scalars: &[ScalarField], points: &[G1Affine]) -> Result<G1Affine, G1ArithmeticError> {
        if scalars.len() > points.len() {
            return Err(G1ArithmeticError);
        }

        Ok(<Bn254 as Pairing>::G1::msm_unchecked(points, scalars).into())
    }
}

fn plain_test(name: &str, proof_file: &str, vk_file: &str, public_inputs_file: &str) {
    // parse proof file
    let proof_u8 = std::fs::read(proof_file).unwrap();
    let proof = HonkProof::from_buffer(&proof_u8).unwrap();

    // parse public_inputs file
    let public_inputs_u8 = std::fs::read(public_inputs_file).unwrap();
    let public_inputs = Vec::<ScalarField>::deserialize_from_bytes(&public_inputs_u8).unwrap();

    // parse the crs
    let verifier_crs = CrsParser::get_crs_g2().unwrap();

    // parse verification key file
    let vk_u8 = std::fs::read(vk_file).unwrap();
    let vk = VerifyingKeyBarretenberg::from_buffer(&vk_u8).unwrap();
    let vk = VerifyingKey::from_barrettenberg_and_crs(vk, verifier_crs);

    let is_valid = UltraHonk::<ArkHonkCurve, ArkKeccak256>::verify(
        proof,
        &public_inputs,
        &vk
    )
    .unwrap();

    assert!(is_valid, "Failed for: {}", name);
}

#[test]
fn test_iterating_test_vectors() {
    let test_vectors_dir = "../../test_vectors";
    let test_vectors = std::fs::read_dir(test_vectors_dir).unwrap();

    println!("test_vectors_dir: {}", test_vectors_dir);

    for entry in test_vectors {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_dir() {
            // Skip if not a directory
            continue;
        }
        let proof_file = format!("{}/kat/proof", path.display());
        let vk_file = format!("{}/kat/vk", path.display());
        let public_inputs_file = format!("{}/kat/public_inputs", path.display());

        plain_test(
            &path.display().to_string(),
            &proof_file,
            &vk_file,
            &public_inputs_file,
        );
    }
}
