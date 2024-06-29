use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher as PH, SaltString},
    Argon2, PasswordVerifier,
};
use async_trait::async_trait;
use colette_core::password::{Error, PasswordHasher};

#[derive(Debug, Default)]
pub struct ArgonHasher<'a> {
    argon2: Argon2<'a>,
}

#[async_trait]
impl PasswordHasher for ArgonHasher<'_> {
    async fn hash(&self, password: &str) -> Result<String, Error> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| Error::Hash)?
            .to_string();

        Ok(password_hash)
    }

    async fn verify(&self, password: &str, hashed: &str) -> Result<bool, Error> {
        let hash = PasswordHash::new(hashed).map_err(|_| Error::Verify)?;

        let valid = self
            .argon2
            .verify_password(password.as_bytes(), &hash)
            .is_ok();

        Ok(valid)
    }
}
