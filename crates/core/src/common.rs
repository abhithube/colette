pub const PAGINATION_LIMIT: u64 = 24;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub cursor: Option<String>,
}

#[async_trait::async_trait]
pub trait Creatable: Send + Sync {
    type Data;
    type Output;

    async fn create(&self, data: Self::Data) -> Self::Output;
}
