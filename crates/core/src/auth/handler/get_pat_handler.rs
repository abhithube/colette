use crate::{
    Handler,
    auth::{PatError, PatId, PatRepository, PersonalAccessToken, UserId},
    common::RepositoryError,
};

#[derive(Debug, Clone)]
pub struct GetPatQuery {
    pub id: PatId,
    pub user_id: UserId,
}

pub struct GetPatHandler {
    pat_repository: Box<dyn PatRepository>,
}

impl GetPatHandler {
    pub fn new(pat_repository: impl PatRepository) -> Self {
        Self {
            pat_repository: Box::new(pat_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<GetPatQuery> for GetPatHandler {
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
