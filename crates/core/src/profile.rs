use std::sync::Arc;

use url::Url;
use uuid::Uuid;

use crate::common::{Creatable, Deletable, Findable, NonEmptyString, Paginated, Updatable};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    pub id: Uuid,
    pub title: String,
    pub image_url: Option<String>,
    pub is_default: bool,
    pub user_id: Uuid,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProfileCreate {
    pub title: NonEmptyString,
    pub image_url: Option<Url>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProfileUpdate {
    pub title: Option<NonEmptyString>,
    pub image_url: Option<Option<Url>>,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub title: String,
}

pub struct ProfileService {
    repository: Arc<dyn ProfileRepository>,
}

impl ProfileService {
    pub fn new(repository: Arc<dyn ProfileRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_profiles(&self, user_id: Uuid) -> Result<Paginated<Profile>, Error> {
        let profiles = self.repository.list(user_id, None, None).await?;

        Ok(Paginated {
            data: profiles,
            ..Default::default()
        })
    }

    pub async fn get_profile(&self, id: Uuid, user_id: Uuid) -> Result<Profile, Error> {
        self.repository
            .find(ProfileIdOrDefaultParams {
                id: Some(id),
                user_id,
            })
            .await
    }

    pub async fn create_profile(
        &self,
        data: ProfileCreate,
        user_id: Uuid,
    ) -> Result<Profile, Error> {
        self.repository
            .create(ProfileCreateData {
                title: data.title.into(),
                image_url: data.image_url.map(String::from),
                user_id,
            })
            .await
    }

    pub async fn update_profile(
        &self,
        id: Uuid,
        data: ProfileUpdate,
        user_id: Uuid,
    ) -> Result<Profile, Error> {
        self.repository
            .update(ProfileIdParams::new(id, user_id), data.into())
            .await
    }

    pub async fn delete_profile(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        self.repository
            .delete(ProfileIdParams::new(id, user_id))
            .await
    }
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
        cursor: Option<Cursor>,
    ) -> Result<Vec<Profile>, Error>;
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
    pub image_url: Option<Option<String>>,
}

impl From<ProfileUpdate> for ProfileUpdateData {
    fn from(value: ProfileUpdate) -> Self {
        Self {
            title: value.title.map(String::from),
            image_url: value.image_url.map(|e| e.map(String::from)),
        }
    }
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
