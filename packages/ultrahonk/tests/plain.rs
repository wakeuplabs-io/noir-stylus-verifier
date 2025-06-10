use ark_bn254::Bn254;
use ark_ec::AffineRepr;
use ark_ec::{pairing::Pairing, CurveGroup};
use ark_ff::One;
use ark_ff::{BigInt, Field};
use ark_serialize::CanonicalDeserialize;
use eyre::{anyhow, Result};
use sha3::{Digest, Keccak256};
use std::fs::File;
use std::io::Read;
use std::marker::PhantomData;
use std::path::Path;
use std::str::FromStr;
use ultrahonk::backends::G1ArithmeticBackend;
use ultrahonk::backends::G1ArithmeticError;
use ultrahonk::honk_curve::HonkCurve;
use ultrahonk::keys::verification_key::VerifyingKey;
use ultrahonk::keys::verification_key::VerifyingKeyBarretenberg;
use ultrahonk::prelude::HashBackend;
use ultrahonk::serialize::Serialize as FieldSerialize;
use ultrahonk::serialize::Serialize;
use ultrahonk::types::G1Affine;
use ultrahonk::types::G2Affine;
use ultrahonk::types::ZeroKnowledge;
use ultrahonk::{
    prelude::{HonkProof, UltraHonk},
    types::ScalarField,
};

pub struct ArkKeccak256;

impl HashBackend for ArkKeccak256 {
    fn hash(buffer: Vec<ScalarField>) -> ScalarField {
        // Losing 2 bits of this is not an issue -> we can just reduce mod p
        let vec = Serialize::to_buffer(&buffer, false);
        let mut hasher = Keccak256::default();
        hasher.update(vec);
        let hash_result = hasher.finalize();

        let mut offset = 0;
        Serialize::read_field_element(&hash_result, &mut offset)
    }
}

struct CrsParser<P: Pairing> {
    _marker: PhantomData<P>,
}

impl<P: Pairing> CrsParser<P> {
    fn convert_endianness_inplace(buffer: &mut [u8]) {
        for chunk in buffer.chunks_exact_mut(32) {
            chunk.reverse();
        }
    }

    fn read_transcript_g2(g2_x: &mut P::G2Affine, path: impl AsRef<Path>) -> Result<()> {
        let g2_size = std::mem::size_of::<<P::G2 as CurveGroup>::BaseField>() * 2;

        assert!(std::mem::size_of::<P::G2Affine>() >= g2_size);
        let mut buffer = vec![0; g2_size];

        let file = File::open(path)?;
        let mut file = file.take(g2_size as u64);
        file.read_exact(&mut buffer[..])?;
        Self::convert_endianness_inplace(&mut buffer);
        *g2_x = P::G2Affine::deserialize_uncompressed(&mut &buffer[..])
            .map_err(|e| anyhow!("Failed to deserialize G2Affine from transcript file: {}", e))?;
        Ok(())
    }

    pub fn get_crs_g2(path_g2: impl AsRef<Path>) -> Result<P::G2Affine> {
        let mut g2_x = P::G2Affine::default();
        Self::read_transcript_g2(&mut g2_x, path_g2)?;

        Ok(g2_x)
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
        tracing::trace!("Pairing check");
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

impl HonkCurve for ArkHonkCurve {
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

fn plain_test(proof_file: &str, vk_file: &str, public_inputs_file: &str) {
    const CRS_PATH_G2: &str = "./src/crs/bn254_g2.dat";

    // parse proof file
    let proof_u8 = std::fs::read(&proof_file).unwrap();
    let proof = HonkProof::from_buffer(&proof_u8).unwrap();

    // parse public_inputs file
    let public_inputs_u8 = std::fs::read(&public_inputs_file).unwrap();
    let public_inputs = FieldSerialize::from_buffer(&public_inputs_u8, false).unwrap();

    // parse the crs
    let verifier_crs = CrsParser::<Bn254>::get_crs_g2(CRS_PATH_G2).unwrap();

    // parse verification key file
    let vk_u8 = std::fs::read(&vk_file).unwrap();
    let vk = VerifyingKeyBarretenberg::from_buffer(&vk_u8).unwrap();
    let vk = VerifyingKey::from_barrettenberg_and_crs(vk, verifier_crs);

    let is_valid = UltraHonk::<ArkHonkCurve, ArkKeccak256>::verify(
        proof,
        &public_inputs,
        &vk,
        ZeroKnowledge::No,
    )
    .unwrap();

    assert!(is_valid);
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

        plain_test(&proof_file, &vk_file, &public_inputs_file);
    }
}
