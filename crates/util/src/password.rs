use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

pub fn hash(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(OsRng);

    let hashed = Argon2::default().hash_password(password.as_bytes(), &salt)?;

    Ok(hashed.to_string())
}

pub fn verify(password: &str, hashed: &str) -> Result<bool, Error> {
    let ph = PasswordHash::new(hashed)?;

    let value = Argon2::default()
        .verify_password(password.as_bytes(), &ph)
        .is_ok();

    Ok(value)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Password(#[from] argon2::password_hash::Error),
}
