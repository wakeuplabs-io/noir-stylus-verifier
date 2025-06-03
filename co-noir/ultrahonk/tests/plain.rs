use ark_bn254::Bn254;
use ark_ec::{pairing::Pairing, CurveGroup};
use ark_serialize::CanonicalDeserialize;
use eyre::{anyhow, Result};
use sha3::Keccak256;
use std::fs::File;
use std::io::Read;
use std::marker::PhantomData;
use std::path::Path;
use ultrahonk::builder::Serialize as FieldSerialize;
use ultrahonk::builder::VerifyingKey;
use ultrahonk::builder::VerifyingKeyBarretenberg;
use ultrahonk::types::ZeroKnowledge;
use ultrahonk::{
    prelude::{HonkProof, TranscriptHasher, UltraHonk},
    transcript::TranscriptFieldType,
};

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

fn plain_test<H: TranscriptHasher<TranscriptFieldType>>(
    proof_file: &str,
    vk_file: &str,
    public_inputs_file: &str,
) {
    const CRS_PATH_G2: &str = "../co-builder/src/crs/bn254_g2.dat";

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
    let vk = VerifyingKeyBarretenberg::<Bn254>::from_buffer(&vk_u8).unwrap();
    let vk = VerifyingKey::from_barrettenberg_and_crs(vk, verifier_crs);

    let is_valid =
        UltraHonk::<_, H>::verify(proof, &public_inputs, &vk, ZeroKnowledge::No).unwrap();
    assert!(is_valid);
}

#[test]
fn poseidon_test_keccak256() {
    const PROOF_FILE: &str = "../../test_vectors/poseidon/kat/proof";
    const VK_FILE: &str = "../../test_vectors/poseidon/kat/vk";
    const PUBLIC_INPUTS_FILE: &str = "../../test_vectors/poseidon/kat/public_inputs";

    plain_test::<Keccak256>(PROOF_FILE, VK_FILE, PUBLIC_INPUTS_FILE);
}

#[test]
fn add3_test_keccak256() {
    const PROOF_FILE: &str = "../../test_vectors/add3u64/kat/proof";
    const VK_FILE: &str = "../../test_vectors/add3u64/kat/vk";
    const PUBLIC_INPUTS_FILE: &str = "../../test_vectors/add3u64/kat/public_inputs";

    plain_test::<Keccak256>(PROOF_FILE, VK_FILE, PUBLIC_INPUTS_FILE);
}
