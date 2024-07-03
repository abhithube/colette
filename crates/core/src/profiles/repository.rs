use super::{Error, Profile};
use async_trait::async_trait;

#[async_trait]
pub trait ProfilesRepository {
    async fn find_many(&self, params: ProfileFindManyParams<'_>) -> Result<Vec<Profile>, Error>;

    async fn find_one(&self, params: ProfileFindOneParams<'_>) -> Result<Profile, Error>;

    async fn create(&self, data: ProfileCreateData<'_>) -> Result<Profile, Error>;

    async fn update(
        &self,
        params: ProfileFindByIdParams<'_>,
        data: ProfileUpdateData<'_>,
    ) -> Result<Profile, Error>;

    async fn delete(&self, params: ProfileFindByIdParams<'_>) -> Result<(), Error>;
}

pub struct ProfileFindManyParams<'a> {
    pub user_id: &'a str,
}

pub struct ProfileFindByIdParams<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
}

pub enum ProfileFindOneParams<'a> {
    ById(ProfileFindByIdParams<'a>),
    Default { user_id: &'a str },
}

pub struct ProfileCreateData<'a> {
    pub title: &'a str,
    pub image_url: Option<&'a str>,
    pub user_id: &'a str,
}

pub struct ProfileUpdateData<'a> {
    pub title: Option<&'a str>,
    pub image_url: Option<&'a str>,
}
