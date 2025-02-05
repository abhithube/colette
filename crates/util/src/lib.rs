pub mod base64;
pub mod password;
pub mod thumbnail;

pub trait PasswordHasher: Send + Sync + 'static {
    fn hash(&self, password: &str) -> Result<String, anyhow::Error>;

    fn verify(&self, password: &str, hashed: &str) -> Result<bool, anyhow::Error>;
}

pub trait DataEncoder<T>: Send + Sync + 'static {
    fn encode(&self, data: &T) -> Result<String, anyhow::Error>;

    fn decode(&self, raw: &str) -> Result<T, anyhow::Error>;
}
