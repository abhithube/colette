use futures::stream::BoxStream;
use uuid::Uuid;

use crate::common::{Creatable, Deletable, Findable, Paginated, Updatable};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    pub id: Uuid,
    pub title: String,
    pub image_url: Option<String>,
    pub is_default: bool,
    pub user_id: Uuid,
}

#[async_trait::async_trait]
pub trait ProfileRepository:
    Findable<Params = ProfileIdOrDefaultParams, Output = Result<Profile, Error>>
    + Creatable<Data = ProfileCreateData, Output = Result<Profile, Error>>
    + Updatable<Params = ProfileIdParams, Data = ProfileUpdateData, Output = Result<Profile, Error>>
    + Deletable<Params = ProfileIdParams, Output = Result<(), Error>>
    + Send
    + Sync
{
    async fn list(
        &self,
        user_id: Uuid,
        limit: Option<u64>,
        cursor: Option<String>,
    ) -> Result<Paginated<Profile>, Error>;

    async fn stream(&self, feed_id: i32) -> Result<BoxStream<Result<Uuid, Error>>, Error>;
}

#[derive(Clone, Debug, Default)]
pub struct ProfileIdParams {
    pub id: Uuid,
    pub user_id: Uuid,
}

impl ProfileIdParams {
    pub fn new(id: Uuid, user_id: Uuid) -> Self {
        Self { id, user_id }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProfileIdOrDefaultParams {
    pub id: Option<Uuid>,
    pub user_id: Uuid,
}

#[derive(Clone, Debug, Default)]
pub struct ProfileCreateData {
    pub title: String,
    pub image_url: Option<String>,
    pub user_id: Uuid,
}

#[derive(Clone, Debug, Default)]
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
