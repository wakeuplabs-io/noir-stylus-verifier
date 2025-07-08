#![macro_use]
extern crate alloc;

use std::process::Command;

use alloc::vec::Vec;
use ark_bn254::Bn254;
use ark_ff::Zero;
use ark_ec::{pairing::Pairing, AffineRepr, CurveGroup};
use ark_ff::One;
use sha3::{Digest, Keccak256};
use ultrahonk::constants::NUM_BYTES_FELT;
use ultrahonk::types::G1BaseField;
use ultrahonk::{
    backends::{G1ArithmeticBackend, G1ArithmeticError, HashBackend},
    constants::HASH_OUTPUT_SIZE,
    keys::verification_key::VerifyingKey,
    serialize::{BytesDeserializable, BytesSerializable},
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
        if a == G1Affine::identity() {
            return Ok(b);
        } else if b == G1Affine::identity() {
            return Ok(a);
        }

        // Serialize the points
        let mut calldata = [0_u8; NUM_BYTES_FELT * 4];
        calldata[..NUM_BYTES_FELT * 2].copy_from_slice(&serialize_g1_affine_to_bytes(a));
        calldata[NUM_BYTES_FELT * 2..].copy_from_slice(&serialize_g1_affine_to_bytes(b));

        // println!("cast call {} \"0x{}\"", "0x0000000000000000000000000000000000000006", hex::encode(calldata));

        let output = Command::new("cast")
            .args(["call", "0x0000000000000000000000000000000000000006", &format!("0x{}", hex::encode(calldata))])
            .output()
            .expect("Failed to execute cast");

        if !output.status.success() {
            eprintln!(
                "Cast call failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(G1ArithmeticError);
        } else  {
            let stdout = String::from_utf8_lossy(&output.stdout);   
            let call_result = stdout.split("0x").nth(1).unwrap().trim();
            let out = hex::decode(call_result).unwrap();

            // let result = deserialize_from_bytes(&out);
            let result = G1Affine::deserialize_from_bytes(&out).unwrap();
           return Ok(result)
        } 

        // Ok((a + b).into_affine())
    }

    /// Multiply a G1 point by a scalar in its scalar field
    fn ec_scalar_mul(a: ScalarField, b: G1Affine) -> Result<G1Affine, G1ArithmeticError> {
        let mut b_group = b.into_group();
        b_group *= a;
        // Ok(b_group.into_affine())


        if a == ScalarField::one() {
            return Ok(b);
        }

        // Serialize the point and scalar
        let mut calldata = [0_u8; NUM_BYTES_FELT * 3];
        calldata[..NUM_BYTES_FELT * 2].copy_from_slice(&b.serialize_to_bytes());
        calldata[NUM_BYTES_FELT * 2..].copy_from_slice(&a.serialize_to_bytes());

        let output = Command::new("cast")
            .args(["call", "0x0000000000000000000000000000000000000007", &format!("0x{}", hex::encode(calldata))])
            .output()
            .expect("Failed to execute cast");
        println!("cast call 0x0000000000000000000000000000000000000007 \"0x{}\"", hex::encode(calldata));

        if !output.status.success() {
            eprintln!("Cast call failed: {}", String::from_utf8_lossy(&output.stderr));
            return Err(G1ArithmeticError);
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);   
            let call_result = stdout.split("0x").nth(1).unwrap().trim();

            println!("\nexpected: {:?}", hex::encode(b_group.into_affine().serialize_to_bytes()));
            println!("call_result: {:?}\n", call_result);

            let out = hex::decode(call_result).unwrap();
            let result = G1Affine::deserialize_from_bytes(&out).unwrap();
            return Ok(result);
        }
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

        println!("scalars: {:?}", scalars.len());
        println!("points: {:?}", points.len());

        scalars
            .iter()
            .zip(points.iter())
            .try_fold(G1Affine::identity(), |acc, (scalar, point)| {
                let scaled_point = Self::ec_scalar_mul(*scalar, *point)?;
                Self::ec_add(acc, scaled_point)
            })
    }
}

fn plain_test(name: &str, proof_file: &str, vk_file: &str, public_inputs_file: &str) {
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

    assert!(is_valid, "Failed for: {}", name);
}

// TODO: use macro to divide into multiple tests

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

        break;
    }
}



fn deserialize_cursor<D: BytesDeserializable>(
    bytes: &[u8],
    cursor: &mut usize,
) -> D {
    let elem = D::deserialize_from_bytes(&bytes[*cursor..*cursor + D::SER_LEN]).unwrap();
    *cursor += D::SER_LEN;
    elem
}

fn deserialize_g1_affine_from_bytes(bytes: &[u8]) -> G1Affine {
    // Note: although this performs modular reduction, it's safe to do so
    // since we can assume that precompiles will always correctly return
    // elements contained in the field
    let mut cursor = 0;
    let x = G1BaseField::deserialize_from_bytes_with_offset(bytes, &mut cursor).unwrap();
    let y = G1BaseField::deserialize_from_bytes_with_offset(bytes, &mut cursor).unwrap();

    G1Affine { x, y, infinity: x.is_zero() && y.is_zero() }
}

fn serialize_g1_affine_to_bytes(d: G1Affine) -> Vec<u8> {
    let zero = G1BaseField::zero();
    let (x, y) = d.xy().unwrap_or((zero, zero));
    let mut bytes = Vec::with_capacity(NUM_BYTES_FELT * 2);
    bytes.extend(x.serialize_to_bytes());
    bytes.extend(y.serialize_to_bytes());
    bytes
}