use base64::{Engine, engine::general_purpose};
use rand::RngCore;

use crate::common::sha256;

pub fn generate() -> String {
    let mut raw = [0; 32];
    rand::rng().fill_bytes(&mut raw);

    general_purpose::STANDARD_NO_PAD.encode(raw)
}

pub fn hash(value: &str) -> String {
    sha256(value)
}
