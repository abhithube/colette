use dyn_clone::DynClone;
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

#[derive(Clone)]
pub struct ProfileService {
    repository: Box<dyn ProfileRepository>,
}

impl ProfileService {
    pub fn new(repository: Box<dyn ProfileRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_profiles(&self, user_id: Uuid) -> Result<Paginated<Profile>, Error> {
        let profiles = self
            .repository
            .find(ProfileFindParams {
                user_id,
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: profiles,
            ..Default::default()
        })
    }

    pub async fn get_profile(&self, id: Uuid, user_id: Uuid) -> Result<Profile, Error> {
        let mut profiles = self
            .repository
            .find(ProfileFindParams {
                id: Some(id),
                user_id,
                ..Default::default()
            })
            .await?;
        if profiles.is_empty() {
            return Err(Error::NotFound(id));
        }

        Ok(profiles.swap_remove(0))
    }

    pub async fn create_profile(
        &self,
        data: ProfileCreate,
        user_id: Uuid,
    ) -> Result<Profile, Error> {
        let id = self
            .repository
            .create(ProfileCreateData {
                title: data.title.into(),
                image_url: data.image_url.map(String::from),
                user_id,
            })
            .await?;

        self.get_profile(id, user_id).await
    }

    pub async fn update_profile(
        &self,
        id: Uuid,
        data: ProfileUpdate,
        user_id: Uuid,
    ) -> Result<Profile, Error> {
        self.repository
            .update(ProfileIdParams::new(id, user_id), data.into())
            .await?;

        self.get_profile(id, user_id).await
    }

    pub async fn delete_profile(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        self.repository
            .delete(ProfileIdParams::new(id, user_id))
            .await
    }
}

pub trait ProfileRepository:
    Findable<Params = ProfileFindParams, Output = Result<Vec<Profile>, Error>>
    + Creatable<Data = ProfileCreateData, Output = Result<Uuid, Error>>
    + Updatable<Params = ProfileIdParams, Data = ProfileUpdateData, Output = Result<(), Error>>
    + Deletable<Params = ProfileIdParams, Output = Result<(), Error>>
    + Send
    + Sync
    + DynClone
{
}

dyn_clone::clone_trait_object!(ProfileRepository);

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
pub struct ProfileFindParams {
    pub id: Option<Uuid>,
    pub is_default: Option<bool>,
    pub user_id: Uuid,
    pub limit: Option<u64>,
    pub cursor: Option<Cursor>,
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
