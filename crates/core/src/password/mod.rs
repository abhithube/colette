mod error;

use async_trait::async_trait;
pub use error::Error;

#[async_trait]
pub trait PasswordHasher {
    async fn hash(&self, password: &str) -> Result<String, Error>;

    async fn verify(&self, password: &str, hashed: &str) -> Result<bool, Error>;
}
