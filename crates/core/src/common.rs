use uuid::Uuid;

pub const PAGINATION_LIMIT: u64 = 24;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct IdParams {
    pub id: Uuid,
    pub user_id: Uuid,
}

impl IdParams {
    pub fn new(id: Uuid, user_id: Uuid) -> Self {
        Self { id, user_id }
    }
}

#[async_trait::async_trait]
pub trait Findable {
    type Params;
    type Output;

    async fn find(&self, params: Self::Params) -> Self::Output;
}

#[async_trait::async_trait]
pub trait Creatable {
    type Data;
    type Output;

    async fn create(&self, data: Self::Data) -> Self::Output;
}

#[async_trait::async_trait]
pub trait Updatable {
    type Params;
    type Data;
    type Output;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output;
}

#[async_trait::async_trait]
pub trait Deletable {
    type Params;
    type Output;

    async fn delete(&self, params: Self::Params) -> Self::Output;
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("cannot be empty")]
    Empty,
}
