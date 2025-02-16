use base64::{Engine, engine::general_purpose};
use sha2::{Digest, Sha256};

pub fn base64(value: &str) -> String {
    general_purpose::STANDARD_NO_PAD.encode(value)
}

pub fn sha256(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());

    hex::encode(hasher.finalize())
}
