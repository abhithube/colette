use dyn_clone::DynClone;

pub mod base64;
pub mod password;

pub trait PasswordHasher: Send + Sync + DynClone {
    fn hash(&self, password: &str) -> Result<String, anyhow::Error>;

    fn verify(&self, password: &str, hashed: &str) -> Result<bool, anyhow::Error>;
}

dyn_clone::clone_trait_object!(PasswordHasher);

pub trait DataEncoder<T>: Send + Sync + DynClone {
    fn encode(&self, data: &T) -> Result<String, anyhow::Error>;

    fn decode(&self, raw: &str) -> Result<T, anyhow::Error>;
}

dyn_clone::clone_trait_object!(<T> DataEncoder<T>);
