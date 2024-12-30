use dyn_clone::DynClone;

#[cfg(feature = "cloudflare")]
pub mod cloudflare;
#[cfg(feature = "in-memory")]
pub mod memory;

#[async_trait::async_trait]
pub trait Queue: Send + Sync + DynClone + 'static {
    type Data;

    async fn push(&self, data: Self::Data) -> Result<(), anyhow::Error>;
}

dyn_clone::clone_trait_object!(<T> Queue<Data = T>);
