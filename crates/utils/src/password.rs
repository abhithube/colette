use anyhow::anyhow;
use password_auth::{verify_password, VerifyError};

pub async fn hash(password: &str) -> Result<String, anyhow::Error> {
    Ok(password_auth::generate_hash(password))
}

pub async fn verify(password: &str, hashed: &str) -> Result<bool, anyhow::Error> {
    match verify_password(password, hashed) {
        Ok(()) => Ok(true),
        Err(VerifyError::PasswordInvalid) => Ok(false),
        _ => Err(anyhow!("couldn't verify password")),
    }
}
