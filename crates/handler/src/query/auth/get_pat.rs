use colette_core::{
    auth::{PatError, PatId, PatRepository, PersonalAccessToken, UserId},
    common::RepositoryError,
};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct GetPatQuery {
    pub id: PatId,
    pub user_id: UserId,
}

pub struct GetPatHandler<PR: PatRepository> {
    pat_repository: PR,
}

impl<PR: PatRepository> GetPatHandler<PR> {
    pub fn new(pat_repository: PR) -> Self {
        Self { pat_repository }
    }
}

#[async_trait::async_trait]
impl<PR: PatRepository> Handler<GetPatQuery> for GetPatHandler<PR> {
    type Response = PersonalAccessToken;
    type Error = GetPatError;

    async fn handle(&self, query: GetPatQuery) -> Result<Self::Response, Self::Error> {
        let pat = self
            .pat_repository
            .find_by_id(query.id, query.user_id)
            .await?
            .ok_or(PatError::NotFound(query.id))?;

        Ok(pat)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetPatError {
    #[error(transparent)]
    Pat(#[from] PatError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
