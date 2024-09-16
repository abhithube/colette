pub mod base64;
pub mod password;

pub trait PasswordHasher: Send + Sync {
    fn hash(&self, password: &str) -> Result<String, anyhow::Error>;

    fn verify(&self, password: &str, hashed: &str) -> Result<bool, anyhow::Error>;
}

pub trait DataEncoder<T>: Send + Sync {
    fn encode(&self, data: &T) -> Result<String, anyhow::Error>;

    fn decode(&self, raw: &str) -> Result<T, anyhow::Error>;
}
