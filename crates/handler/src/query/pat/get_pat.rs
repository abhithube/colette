use colette_core::{common::RepositoryError, pat::PatError};
use uuid::Uuid;

use crate::{Handler, PatQueryRepository, PersonalAccessTokenDto};

#[derive(Debug, Clone)]
pub struct GetPatQuery {
    pub id: Uuid,
    pub user_id: Uuid,
}

pub struct GetPatHandler<PQR: PatQueryRepository> {
    pat_query_repository: PQR,
}

impl<PQR: PatQueryRepository> GetPatHandler<PQR> {
    pub fn new(pat_query_repository: PQR) -> Self {
        Self {
            pat_query_repository,
        }
    }
}

#[async_trait::async_trait]
impl<PQR: PatQueryRepository> Handler<GetPatQuery> for GetPatHandler<PQR> {
    type Response = PersonalAccessTokenDto;
    type Error = GetPatError;

    async fn handle(&self, query: GetPatQuery) -> Result<Self::Response, Self::Error> {
        let pat = self
            .pat_query_repository
            .query_by_id(query.id, query.user_id)
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
