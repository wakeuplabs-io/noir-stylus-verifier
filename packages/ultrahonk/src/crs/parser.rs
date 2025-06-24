use crate::serialize::BytesDeserializable;
use crate::types::{G2Affine, HonkProofError};

pub struct CrsParser {}

impl CrsParser {
    pub fn get_crs_g2() -> Result<G2Affine, HonkProofError> {
        let g2_x = G2Affine::deserialize_from_bytes(hex::decode("260e01b251f6f1c7e7ff4e580791dee8ea51d87a358e038b4efe30fac09383c10118c4d5b837bcc2bc89b5b398b5974e9f5944073b32078b7e231fec938883b004fc6369f7110fe3d25156c1bb9a72859cf2a04641f99ba4ee413c80da6a5fe422febda3c0c0632a56475b4214e5615e11e6dd3f96e6cea2854a87d4dacc5e55").unwrap().as_slice()).unwrap();

        Ok(g2_x)
    }
}
