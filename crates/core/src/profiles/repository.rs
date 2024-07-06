use async_trait::async_trait;

use super::{Error, Profile};

#[async_trait]
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
