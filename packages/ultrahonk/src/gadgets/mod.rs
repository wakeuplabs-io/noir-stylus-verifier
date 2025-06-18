pub mod poseidon2;
use ark_ff::PrimeField;

/// Reads a field elemnent from a hexadecimal string. Therebey, the format can or can not include the 0x prefix, i.e., "0x2" and "2" give the same result.
pub fn field_from_hex_string<F: PrimeField>(str: &str) -> Result<F, &'static str> {

    let s = str.strip_prefix("0x").unwrap_or(str);
    let bytes = hex::decode(s).unwrap();
    Ok(F::from_be_bytes_mod_order(&bytes))
}
