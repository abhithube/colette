use std::str::Utf8Error;

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use base64::{DecodeError, Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
use rand::RngCore as _;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub fn argon2_hash(value: &str) -> Result<String, Argon2Error> {
    let salt = SaltString::generate(OsRng);

    let hashed = Argon2::default().hash_password(value.as_bytes(), &salt)?;

    Ok(hashed.to_string())
}

pub fn argon2_verify(value: &str, hashed: &str) -> Result<bool, Argon2Error> {
    let ph = PasswordHash::new(hashed)?;

    let value = Argon2::default()
        .verify_password(value.as_bytes(), &ph)
        .is_ok();

    Ok(value)
}

#[derive(Debug, thiserror::Error)]
pub enum Argon2Error {
    #[error(transparent)]
    Hash(#[from] argon2::password_hash::Error),
}

pub fn base64_encode<T: Serialize>(data: &T) -> Result<String, Base64Error> {
    let raw = serde_json::to_string(data)?;

    Ok(BASE64_URL_SAFE_NO_PAD.encode(&raw))
}

pub fn base64_decode<T: for<'a> Deserialize<'a>>(raw: &str) -> Result<T, Base64Error> {
    let decoded = BASE64_URL_SAFE_NO_PAD.decode(raw)?;
    let data_str = str::from_utf8(&decoded)?;
    let data = serde_json::from_str::<T>(data_str)?;

    Ok(data)
}

#[derive(Debug, thiserror::Error)]
pub enum Base64Error {
    #[error(transparent)]
    Utf(#[from] Utf8Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Decode(#[from] DecodeError),
}

pub fn random_generate(len: usize) -> String {
    let mut raw = vec![0; len];
    rand::rng().fill_bytes(&mut raw);

    BASE64_URL_SAFE_NO_PAD.encode(raw)
}

pub fn sha256_hash(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());

    hex::encode(hasher.finalize())
}
