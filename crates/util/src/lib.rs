use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use rand::RngCore as _;
use sha2::{Digest, Sha256};
use uuid::{ContextV7, NoContext, Timestamp, Uuid};

pub fn uuid_generate() -> Uuid {
    Uuid::now_v7()
}

pub fn uuid_generate_ts(now: DateTime<Utc>) -> Uuid {
    Uuid::new_v7(Timestamp::from_unix(
        NoContext,
        now.timestamp() as u64,
        now.timestamp_subsec_nanos(),
    ))
}

pub fn uuid_generate_ctx(ctx: ContextV7, now: DateTime<Utc>) -> Uuid {
    Uuid::new_v7(Timestamp::from_unix(
        ctx,
        now.timestamp() as u64,
        now.timestamp_subsec_nanos(),
    ))
}

pub fn argon2_hash(value: &str) -> Result<String, CryptoError> {
    let salt = SaltString::generate(&mut OsRng);
    let hashed = Argon2::default().hash_password(value.as_bytes(), &salt)?;

    Ok(hashed.to_string())
}

pub fn argon2_verify(value: &str, hashed: &str) -> Result<bool, CryptoError> {
    let ph = PasswordHash::new(hashed)?;
    let value = Argon2::default()
        .verify_password(value.as_bytes(), &ph)
        .is_ok();

    Ok(value)
}

pub fn base64_encode(data: &[u8]) -> String {
    general_purpose::STANDARD_NO_PAD.encode(data)
}

pub fn base64_decode(raw: &str) -> Result<Vec<u8>, CryptoError> {
    general_purpose::STANDARD_NO_PAD
        .decode(raw)
        .map_err(CryptoError::Base64)
}

pub fn base64_url_encode(data: &[u8]) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(data)
}

pub fn random_generate(len: usize) -> Vec<u8> {
    let mut data = vec![0; len];
    rand::rng().fill_bytes(&mut data);

    data
}

pub fn sha256_hash(value: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());

    hasher.finalize().to_vec()
}

pub fn hex_encode(data: &[u8]) -> String {
    hex::encode(data)
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error(transparent)]
    Argon2(#[from] argon2::password_hash::Error),

    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
}
