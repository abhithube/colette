use async_trait::async_trait;
use colette_core::utils::password::{Error, PasswordHasher};
use password_auth::{generate_hash, verify_password, VerifyError};

#[derive(Default)]
pub struct Argon2Hasher {}

#[async_trait]
impl PasswordHasher for Argon2Hasher {
    async fn hash(&self, password: &str) -> Result<String, Error> {
        Ok(generate_hash(password))
    }

    async fn verify(&self, password: &str, hashed: &str) -> Result<bool, Error> {
        match verify_password(password, hashed) {
            Ok(()) => Ok(true),
            Err(VerifyError::PasswordInvalid) => Ok(false),
            Err(_) => Err(Error::Verify),
        }
    }
}
