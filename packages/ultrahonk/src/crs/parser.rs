use core::marker::PhantomData;
use ark_ec::{pairing::Pairing, CurveGroup};
use ark_serialize::CanonicalDeserialize;

use crate::types::HonkProofError;

pub struct CrsParser<P: Pairing> {
    _marker: PhantomData<P>,
}

impl<P: Pairing> CrsParser<P> {
    fn convert_endianness_inplace(buffer: &mut [u8]) {
        for chunk in buffer.chunks_exact_mut(32) {
            chunk.reverse();
        }
    }

    fn read_transcript_g2(g2_x: &mut P::G2Affine) -> Result<(), HonkProofError> {
        let g2_size = core::mem::size_of::<<P::G2 as CurveGroup>::BaseField>() * 2;

        assert!(core::mem::size_of::<P::G2Affine>() >= g2_size);
        let mut buffer = vec![0; g2_size];

        buffer.copy_from_slice(&include_bytes!("./bn254_g2.dat")[..g2_size]);
        Self::convert_endianness_inplace(&mut buffer);
        *g2_x = P::G2Affine::deserialize_uncompressed(&mut &buffer[..])
            .map_err(|_| HonkProofError::DeserializationError())?;
        Ok(())
    }

    pub fn get_crs_g2() -> Result<P::G2Affine, HonkProofError> {
        let mut g2_x = P::G2Affine::default();
        Self::read_transcript_g2(&mut g2_x)?;

        Ok(g2_x)
    }
}