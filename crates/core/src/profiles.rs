use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::common::{Paginated, Session};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("profile not found with id: {0}")]
    NotFound(String),

    #[error("default profile cannot be deleted")]
    DeletingDefault,

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug)]
pub struct Profile {
    pub id: String,
    pub title: String,
    pub image_url: Option<String>,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CreateProfile {
    pub title: String,
    pub image_url: Option<String>,
}

#[derive(Debug)]
pub struct UpdateProfile {
    pub title: Option<String>,
    pub image_url: Option<String>,
}

impl From<UpdateProfile> for ProfileUpdateData {
    fn from(value: UpdateProfile) -> Self {
        Self {
            title: value.title,
            image_url: value.image_url,
        }
    }
}

#[async_trait::async_trait]
pub trait ProfilesRepository {
    async fn find_many(&self, params: ProfileFindManyParams) -> Result<Vec<Profile>, Error>;

    async fn find_one(&self, params: ProfileFindOneParams) -> Result<Profile, Error>;

    async fn create(&self, data: ProfileCreateData) -> Result<Profile, Error>;

    async fn update(
        &self,
        params: ProfileFindByIdParams,
        data: ProfileUpdateData,
    ) -> Result<Profile, Error>;

    async fn delete(&self, params: ProfileFindByIdParams) -> Result<(), Error>;
}

pub struct ProfileFindManyParams {
    pub user_id: String,
}

pub struct ProfileFindByIdParams {
    pub id: String,
    pub user_id: String,
}

pub enum ProfileFindOneParams {
    ById(ProfileFindByIdParams),
    Default { user_id: String },
}

pub struct ProfileCreateData {
    pub title: String,
    pub image_url: Option<String>,
    pub user_id: String,
}

pub struct ProfileUpdateData {
    pub title: Option<String>,
    pub image_url: Option<String>,
}

pub struct ProfilesService {
    repo: Arc<dyn ProfilesRepository + Send + Sync>,
}

impl ProfilesService {
    pub fn new(repo: Arc<dyn ProfilesRepository + Send + Sync>) -> Self {
        Self { repo }
    }

    pub async fn list(&self, session: Session) -> Result<Paginated<Profile>, Error> {
        let params = ProfileFindManyParams {
            user_id: session.user_id,
        };
        let profiles = self.repo.find_many(params).await?;

        let paginated = Paginated::<Profile> {
            has_more: false,
            data: profiles,
        };

        Ok(paginated)
    }

    pub async fn get(&self, id: String, session: Session) -> Result<Profile, Error> {
        let params = ProfileFindByIdParams {
            id,
            user_id: session.user_id,
        };
        let params = ProfileFindOneParams::ById(params);
        let profile = self.repo.find_one(params).await?;

        Ok(profile)
    }

    pub async fn create(&self, data: CreateProfile, session: Session) -> Result<Profile, Error> {
        let data = ProfileCreateData {
            title: data.title,
            image_url: data.image_url,
            user_id: session.user_id,
        };
        let profile = self.repo.create(data).await?;

        Ok(profile)
    }

    pub async fn update(
        &self,
        id: String,
        data: UpdateProfile,
        session: Session,
    ) -> Result<Profile, Error> {
        let params = ProfileFindByIdParams {
            id,
            user_id: session.user_id,
        };
        let profile = self.repo.update(params, data.into()).await?;

        Ok(profile)
    }

    pub async fn delete(&self, id: String, session: Session) -> Result<(), Error> {
        let params = ProfileFindByIdParams {
            id,
            user_id: session.user_id,
        };
        self.repo.delete(params).await?;

        Ok(())
    }
}
