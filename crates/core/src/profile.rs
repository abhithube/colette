use futures::stream::BoxStream;
use uuid::Uuid;

use crate::common::{Creatable, Deletable, Paginated};

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
pub trait ProfileRepository:
    Creatable<Data = ProfileCreateData, Output = Result<Profile, Error>>
    + Deletable<Params = ProfileIdParams, Output = Result<(), Error>>
    + Send
    + Sync
{
    async fn find_many(
        &self,
        user_id: Uuid,
        limit: Option<u64>,
        cursor: Option<String>,
    ) -> Result<Paginated<Profile>, Error>;

    async fn find_one(&self, id: ProfileIdOrDefaultParams) -> Result<Profile, Error>;

    async fn update(
        &self,
        params: ProfileIdParams,
        data: ProfileUpdateData,
    ) -> Result<Profile, Error>;

    async fn stream(&self, feed_id: i32) -> Result<BoxStream<Result<StreamProfile, Error>>, Error>;
}

#[derive(Clone, Debug)]
pub struct ProfileIdParams {
    pub id: Uuid,
    pub user_id: Uuid,
}

impl ProfileIdParams {
    pub fn new(id: Uuid, user_id: Uuid) -> Self {
        Self { id, user_id }
    }
}

#[derive(Clone, Debug)]
pub struct ProfileIdOrDefaultParams {
    pub id: Option<Uuid>,
    pub user_id: Uuid,
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
