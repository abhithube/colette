use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::common::{FindOneParams, Paginated, Session};

#[derive(Debug)]
pub struct Collection {
    pub id: Uuid,
    pub title: String,
    pub profile_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub bookmark_count: Option<i64>,
}

#[derive(Debug)]
pub struct CreateCollection {
    pub title: String,
}

#[async_trait::async_trait]
pub trait CollectionsRepository {
    async fn find_many(&self, params: CollectionFindManyParams) -> Result<Vec<Collection>, Error>;

    async fn find_one(&self, params: FindOneParams) -> Result<Collection, Error>;

    async fn create(&self, data: CollectionCreateData) -> Result<Collection, Error>;

    async fn delete(&self, params: FindOneParams) -> Result<(), Error>;
}

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

    pub async fn get(&self, id: Uuid, session: Session) -> Result<Collection, Error> {
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

    pub async fn delete(&self, id: Uuid, session: Session) -> Result<(), Error> {
        let params = FindOneParams {
            id,
            profile_id: session.profile_id,
        };
        self.repo.delete(params).await?;

        Ok(())
    }
}

pub struct CollectionFindManyParams {
    pub profile_id: Uuid,
}

pub struct CollectionCreateData {
    pub title: String,
    pub profile_id: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("collection not found with id: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
