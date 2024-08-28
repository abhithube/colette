pub mod base_64;
pub mod password;

pub trait PasswordHasher: Send + Sync {
    fn hash(&self, password: &str) -> Result<String, anyhow::Error>;

    fn verify(&self, password: &str, hashed: &str) -> Result<bool, anyhow::Error>;
}

pub fn hash(password: &str) -> Result<String, anyhow::Error> {
    Ok(password_auth::generate_hash(password))
}
