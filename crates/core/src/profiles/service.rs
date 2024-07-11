use std::sync::Arc;

use super::{
    CreateProfile, Error, ProfileCreateData, ProfileFindByIdParams, ProfileFindManyParams,
    ProfileFindOneParams, ProfilesRepository, UpdateProfile,
};
use crate::{
    common::{Paginated, Session},
    Profile,
};

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
