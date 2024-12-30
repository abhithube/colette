#[cfg(feature = "cloudflare")]
pub mod cloudflare;
#[cfg(feature = "in-memory")]
pub mod memory;

#[async_trait::async_trait]
pub trait Queue: Send + Sync + 'static {
    type Data;

    async fn push(&self, data: Self::Data) -> Result<(), anyhow::Error>;
}
