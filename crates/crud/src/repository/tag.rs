use colette_authentication::UserId;
use colette_common::RepositoryError;

use crate::{Tag, TagId};

pub trait TagRepository: Sync {
    fn find_by_id(
        &self,
        id: TagId,
        user_id: UserId,
    ) -> impl Future<Output = Result<Option<Tag>, RepositoryError>> + Send;

    fn save(&self, data: &Tag) -> impl Future<Output = Result<(), RepositoryError>> + Send;

    fn delete_by_id(
        &self,
        id: TagId,
        user_id: UserId,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;
}
