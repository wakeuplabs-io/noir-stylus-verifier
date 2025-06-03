use ark_bn254::Bn254;
use ultrahonk::builder::CrsParser;
use ultrahonk::builder::Serialize as FieldSerialize;
use ultrahonk::builder::VerifyingKeyBarretenberg;
use ultrahonk::builder::ZeroKnowledge;
use sha3::Keccak256;
use ultrahonk::builder::VerifyingKey;
use ultrahonk::{
    builder::{TranscriptFieldType},
    prelude::{HonkProof, TranscriptHasher, UltraHonk},
};

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
        UltraHonk::<_, H>::verify(proof, &public_inputs, &vk, ZeroKnowledge::No)
            .unwrap();
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
