use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::common::{FindOneParams, Paginated, Session};

#[derive(Clone, Debug)]
pub struct Collection {
    pub id: Uuid,
    pub title: String,
    pub profile_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub bookmark_count: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct CreateCollection {
    pub title: String,
}

#[derive(Clone, Debug)]
pub struct UpdateCollection {
    pub title: Option<String>,
}

#[async_trait::async_trait]
pub trait CollectionsRepository: Send + Sync {
    async fn find_many(&self, params: CollectionFindManyParams) -> Result<Vec<Collection>, Error>;

    async fn find_one(&self, params: FindOneParams) -> Result<Collection, Error>;

    async fn create(&self, data: CollectionCreateData) -> Result<Collection, Error>;

    async fn update(
        &self,
        params: FindOneParams,
        data: CollectionUpdateData,
    ) -> Result<Collection, Error>;

    async fn delete(&self, params: FindOneParams) -> Result<(), Error>;
}

pub struct CollectionsService {
    repo: Arc<dyn CollectionsRepository>,
}

impl CollectionsService {
    pub fn new(repo: Arc<dyn CollectionsRepository>) -> Self {
        Self { repo }
    }

    pub async fn list(&self, session: Session) -> Result<Paginated<Collection>, Error> {
        let collections = self
            .repo
            .find_many(CollectionFindManyParams {
                profile_id: session.profile_id,
            })
            .await?;

        let paginated = Paginated::<Collection> {
            has_more: false,
            data: collections,
        };

        Ok(paginated)
    }

    pub async fn get(&self, id: Uuid, session: Session) -> Result<Collection, Error> {
        let collection = self
            .repo
            .find_one(FindOneParams {
                id,
                profile_id: session.profile_id,
            })
            .await?;

        Ok(collection)
    }

    pub async fn create(
        &self,
        data: CreateCollection,
        session: Session,
    ) -> Result<Collection, Error> {
        let collections = self
            .repo
            .create(CollectionCreateData {
                title: data.title,
                profile_id: session.profile_id,
            })
            .await?;

        Ok(collections)
    }

    pub async fn update(
        &self,
        id: Uuid,
        data: UpdateCollection,
        session: Session,
    ) -> Result<Collection, Error> {
        let collection = self
            .repo
            .update(
                FindOneParams {
                    id,
                    profile_id: session.profile_id,
                },
                data.into(),
            )
            .await?;

        Ok(collection)
    }

    pub async fn delete(&self, id: Uuid, session: Session) -> Result<(), Error> {
        self.repo
            .delete(FindOneParams {
                id,
                profile_id: session.profile_id,
            })
            .await?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct CollectionFindManyParams {
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct CollectionCreateData {
    pub title: String,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct CollectionUpdateData {
    pub title: Option<String>,
}

impl From<UpdateCollection> for CollectionUpdateData {
    fn from(value: UpdateCollection) -> Self {
        Self { title: value.title }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("collection not found with id: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
