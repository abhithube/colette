use futures::stream::BoxStream;
use uuid::Uuid;

use crate::common::Paginated;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Profile {
    pub id: Uuid,
    pub title: String,
    pub image_url: Option<String>,
    pub is_default: bool,
    pub user_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct StreamProfile {
    pub id: Uuid,
}

#[async_trait::async_trait]
pub trait ProfileRepository: Send + Sync {
    async fn find_many_profiles(
        &self,
        user_id: Uuid,
        limit: Option<u64>,
        cursor: Option<String>,
    ) -> Result<Paginated<Profile>, Error>;

    async fn find_one_profile(&self, id: Option<Uuid>, user_id: Uuid) -> Result<Profile, Error>;

    async fn create_profile(&self, data: ProfileCreateData) -> Result<Profile, Error>;

    async fn update_profile(
        &self,
        id: Uuid,
        user_id: Uuid,
        data: ProfileUpdateData,
    ) -> Result<Profile, Error>;

    async fn delete_profile(&self, id: Uuid, user_id: Uuid) -> Result<(), Error>;

    async fn stream_profiles(
        &self,
        feed_id: i32,
    ) -> Result<BoxStream<Result<StreamProfile, Error>>, Error>;
}

#[derive(Clone, Debug)]
pub struct ProfileCreateData {
    pub title: String,
    pub image_url: Option<String>,
    pub user_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct ProfileUpdateData {
    pub title: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("profile not found with id: {0}")]
    NotFound(Uuid),

    #[error("profile already exists with title: {0}")]
    Conflict(String),

    #[error("default profile cannot be deleted")]
    DeletingDefault,

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
