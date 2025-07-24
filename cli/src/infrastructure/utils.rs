use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

#[cfg_attr(test, mockall::automock)]
pub(crate) trait TSha256Hasher: Send + Sync {
    fn hash(&self, path: &Path) -> std::io::Result<String>;
}

#[derive(Default)]
pub(crate) struct Sha256Hasher;

impl TSha256Hasher for Sha256Hasher {
    fn hash(&self, path: &Path) -> std::io::Result<String> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        Ok(hex::encode(hasher.finalize()))
    }
}
