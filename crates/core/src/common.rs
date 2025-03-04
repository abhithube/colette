use std::any::Any;

use sea_orm::DbErr;

pub const PAGINATION_LIMIT: u64 = 24;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub cursor: Option<String>,
}

#[async_trait::async_trait]
pub trait Transaction: Send + Sync + 'static {
    async fn commit(self: Box<Self>) -> Result<(), DbErr>;

    async fn rollback(self: Box<Self>) -> Result<(), DbErr>;

    fn as_any(&self) -> &dyn Any;
}

#[async_trait::async_trait]
pub trait TransactionManager: Send + Sync + 'static {
    async fn begin(&self) -> Result<Box<dyn Transaction>, DbErr>;
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("cannot be empty")]
    Empty,
}
