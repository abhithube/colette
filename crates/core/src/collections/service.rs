use std::sync::Arc;

use super::{
    model::CreateCollection, CollectionCreateData, CollectionFindManyParams, CollectionsRepository,
    Error,
};
use crate::{
    common::{FindOneParams, Paginated, Session},
    Collection,
};

pub struct CollectionsService {
    repo: Arc<dyn CollectionsRepository + Send + Sync>,
}

impl CollectionsService {
    pub fn new(repo: Arc<dyn CollectionsRepository + Send + Sync>) -> Self {
        Self { repo }
    }

    pub async fn list(&self, session: Session) -> Result<Paginated<Collection>, Error> {
        let params = CollectionFindManyParams {
            profile_id: session.profile_id,
        };
        let collections = self.repo.find_many(params).await?;

        let paginated = Paginated::<Collection> {
            has_more: false,
            data: collections,
        };

        Ok(paginated)
    }

    pub async fn get(&self, id: String, session: Session) -> Result<Collection, Error> {
        let params = FindOneParams {
            id,
            profile_id: session.profile_id,
        };
        let collection = self.repo.find_one(params).await?;

        Ok(collection)
    }

    pub async fn create(
        &self,
        data: CreateCollection,
        session: Session,
    ) -> Result<Collection, Error> {
        let data = CollectionCreateData {
            title: data.title,
            profile_id: session.profile_id,
        };
        let collections = self.repo.create(data).await?;

        Ok(collections)
    }

    pub async fn delete(&self, id: String, session: Session) -> Result<(), Error> {
        let params = FindOneParams {
            id,
            profile_id: session.profile_id,
        };
        self.repo.delete(params).await?;

        Ok(())
    }
}
