use std::sync::Arc;

use futures::stream::BoxStream;
use uuid::Uuid;

use crate::common::{Paginated, Session};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Profile {
    pub id: Uuid,
    pub title: String,
    pub image_url: Option<String>,
    pub is_default: bool,
    pub user_id: Uuid,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct CreateProfile {
    pub title: String,
    pub image_url: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct UpdateProfile {
    pub title: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Clone, Debug)]
pub struct StreamProfile {
    pub id: Uuid,
}

#[async_trait::async_trait]
pub trait ProfilesRepository: Send + Sync {
    async fn find_many_profiles(
        &self,
        user_id: Uuid,
        limit: Option<u64>,
        cursor: Option<String>,
    ) -> Result<Paginated<Profile>, Error>;

    async fn find_one_profile(&self, params: ProfilesFindOneParams) -> Result<Profile, Error>;

    async fn create_profile(&self, data: ProfilesCreateData) -> Result<Profile, Error>;

    async fn update_profile(
        &self,
        params: ProfilesFindByIdParams,
        data: ProfilesUpdateData,
    ) -> Result<Profile, Error>;

    async fn delete_profile(&self, params: ProfilesFindByIdParams) -> Result<(), Error>;

    async fn stream_profiles(
        &self,
        feed_id: i32,
    ) -> Result<BoxStream<Result<StreamProfile, Error>>, Error>;
}

pub struct ProfilesService {
    repo: Arc<dyn ProfilesRepository>,
}

impl ProfilesService {
    pub fn new(repo: Arc<dyn ProfilesRepository>) -> Self {
        Self { repo }
    }

    pub async fn list(&self, session: Session) -> Result<Paginated<Profile>, Error> {
        self.repo
            .find_many_profiles(session.user_id, None, None)
            .await
    }

    pub async fn get(&self, id: Uuid, session: Session) -> Result<Profile, Error> {
        self.repo
            .find_one_profile(ProfilesFindOneParams::ById(ProfilesFindByIdParams {
                id,
                user_id: session.user_id,
            }))
            .await
    }

    pub async fn get_default(&self, session: Session) -> Result<Profile, Error> {
        self.repo
            .find_one_profile(ProfilesFindOneParams::ById(ProfilesFindByIdParams {
                id: session.profile_id,
                user_id: session.user_id,
            }))
            .await
    }

    pub async fn create(&self, data: CreateProfile, session: Session) -> Result<Profile, Error> {
        self.repo
            .create_profile(ProfilesCreateData {
                title: data.title,
                image_url: data.image_url,
                user_id: session.user_id,
            })
            .await
    }

    pub async fn update(
        &self,
        id: Uuid,
        data: UpdateProfile,
        session: Session,
    ) -> Result<Profile, Error> {
        self.repo
            .update_profile(
                ProfilesFindByIdParams {
                    id,
                    user_id: session.user_id,
                },
                data.into(),
            )
            .await
    }

    pub async fn delete(&self, id: Uuid, session: Session) -> Result<(), Error> {
        self.repo
            .delete_profile(ProfilesFindByIdParams {
                id,
                user_id: session.user_id,
            })
            .await
    }
}

#[derive(Clone, Debug)]
pub struct ProfilesFindByIdParams {
    pub id: Uuid,
    pub user_id: Uuid,
}

#[derive(Clone, Debug)]
pub enum ProfilesFindOneParams {
    ById(ProfilesFindByIdParams),
    Default { user_id: Uuid },
}

#[derive(Clone, Debug)]
pub struct ProfilesCreateData {
    pub title: String,
    pub image_url: Option<String>,
    pub user_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct ProfilesUpdateData {
    pub title: Option<String>,
    pub image_url: Option<String>,
}

impl From<UpdateProfile> for ProfilesUpdateData {
    fn from(value: UpdateProfile) -> Self {
        Self {
            title: value.title,
            image_url: value.image_url,
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
