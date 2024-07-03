use std::sync::Arc;

use crate::{
    common::{Paginated, Session},
    Profile,
};

use super::{
    CreateProfileDto, Error, ProfileCreateData, ProfileFindByIdParams, ProfileFindManyParams,
    ProfileFindOneParams, ProfilesRepository, UpdateProfileDto,
};

pub struct ProfilesService {
    profiles_repo: Arc<dyn ProfilesRepository + Send + Sync>,
}

impl ProfilesService {
    pub fn new(profiles_repo: Arc<dyn ProfilesRepository + Send + Sync>) -> Self {
        Self { profiles_repo }
    }

    pub async fn list(&self, session: Session) -> Result<Paginated<Profile>, Error> {
        let params = ProfileFindManyParams {
            user_id: session.user_id.as_str(),
        };
        let profiles = self.profiles_repo.find_many(params).await?;

        let paginated = Paginated::<Profile> {
            has_more: false,
            data: profiles,
        };

        Ok(paginated)
    }

    pub async fn get(&self, id: String, session: Session) -> Result<Profile, Error> {
        let params = ProfileFindByIdParams {
            id: id.as_str(),
            user_id: session.user_id.as_str(),
        };
        let params = ProfileFindOneParams::ById(params);
        let profile = self.profiles_repo.find_one(params).await?;

        Ok(profile)
    }

    pub async fn create(&self, dto: CreateProfileDto, session: Session) -> Result<Profile, Error> {
        let data = ProfileCreateData {
            title: dto.title.as_str(),
            image_url: dto.image_url.as_deref(),
            user_id: session.user_id.as_str(),
        };
        let profile = self.profiles_repo.create(data).await?;

        Ok(profile)
    }

    pub async fn update(
        &self,
        id: String,
        dto: UpdateProfileDto,
        session: Session,
    ) -> Result<Profile, Error> {
        let params = ProfileFindByIdParams {
            id: id.as_str(),
            user_id: session.user_id.as_str(),
        };
        let profile = self.profiles_repo.update(params, (&dto).into()).await?;

        Ok(profile)
    }

    pub async fn delete(&self, id: String, session: Session) -> Result<(), Error> {
        let params = ProfileFindByIdParams {
            id: id.as_str(),
            user_id: session.user_id.as_str(),
        };
        self.profiles_repo.delete(params).await?;

        Ok(())
    }
}
