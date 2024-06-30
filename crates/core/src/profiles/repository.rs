use super::{
    Error, Profile, ProfileCreateData, ProfileFindByIdParams, ProfileFindManyParams,
    ProfileFindOneParams, ProfileUpdateData,
};
use async_trait::async_trait;

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
