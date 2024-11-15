use anyhow::anyhow;
use password_auth::{verify_password, VerifyError};

use crate::PasswordHasher;

#[derive(Debug, Clone)]
pub struct ArgonHasher;

impl PasswordHasher for ArgonHasher {
    fn hash(&self, password: &str) -> Result<String, anyhow::Error> {
        Ok(password_auth::generate_hash(password))
    }

    fn verify(&self, password: &str, hashed: &str) -> Result<bool, anyhow::Error> {
        match verify_password(password, hashed) {
            Ok(()) => Ok(true),
            Err(VerifyError::PasswordInvalid) => Ok(false),
            _ => Err(anyhow!("couldn't verify password")),
        }
    }
}
