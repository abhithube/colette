use uuid::Uuid;

pub const PAGINATION_LIMIT: u64 = 24;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub cursor: Option<String>,
}

#[derive(Clone, Debug)]
pub struct IdParams {
    pub id: Uuid,
    pub profile_id: Uuid,
}

impl IdParams {
    pub fn new(id: Uuid, profile_id: Uuid) -> Self {
        Self { id, profile_id }
    }
}

#[async_trait::async_trait]
pub trait Creatable: Send + Sync {
    type Data;
    type Output;

    async fn create(&self, data: Self::Data) -> Self::Output;
}
