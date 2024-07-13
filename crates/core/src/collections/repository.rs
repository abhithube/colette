use async_trait::async_trait;

use super::{Collection, Error};
use crate::common::FindOneParams;

#[async_trait]
pub trait CollectionsRepository {
    async fn find_many(&self, params: CollectionFindManyParams) -> Result<Vec<Collection>, Error>;

    async fn find_one(&self, params: FindOneParams) -> Result<Collection, Error>;

    async fn create(&self, data: CollectionCreateData) -> Result<Collection, Error>;

    async fn delete(&self, params: FindOneParams) -> Result<(), Error>;
}

pub struct CollectionFindManyParams {
    pub profile_id: String,
}

pub struct CollectionCreateData {
    pub title: String,
    pub profile_id: String,
}
