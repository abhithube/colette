#[async_trait::async_trait]
pub trait PasswordHasher: Send + Sync {
    async fn hash(&self, password: &str) -> Result<String, Error>;

    async fn verify(&self, password: &str, hashed: &str) -> Result<bool, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to hash password")]
    Hash,

    #[error("failed to verify password")]
    Verify,
}
