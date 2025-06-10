use ark_bn254::Bn254;
use ark_ec::{pairing::Pairing, CurveGroup};
use ark_serialize::CanonicalDeserialize;
use eyre::{anyhow, Result};
use sha3::{Digest, Keccak256};
use ultrahonk::honk_curve::HonkCurve;
use std::fs::File;
use std::io::Read;
use std::marker::PhantomData;
use std::path::Path;
use ultrahonk::keys::verification_key::VerifyingKey;
use ultrahonk::keys::verification_key::VerifyingKeyBarretenberg;
use ultrahonk::prelude::HashBackend;
use ultrahonk::serialize::Serialize as FieldSerialize;
use ultrahonk::serialize::Serialize;
use ultrahonk::types::ZeroKnowledge;
use ultrahonk::{
    prelude::{HonkProof, UltraHonk},
    types::ScalarField,
};
use ark_ff::{BigInt, Field};
use std::str::FromStr;

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




fn plain_test<H: HashBackend>(proof_file: &str, vk_file: &str, public_inputs_file: &str) {
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

    let is_valid =
        UltraHonk::<ArkHonkCurve, H>::verify(proof, &public_inputs, &vk, ZeroKnowledge::No).unwrap();
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

        plain_test::<ArkKeccak256>(&proof_file, &vk_file, &public_inputs_file);
    }
}
