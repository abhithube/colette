use colette_core::{
    auth::UserId,
    common::RepositoryError,
    pat::{PatError, PatId, PatRepository, PatTitle, PersonalAccessToken},
};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct UpdatePatCommand {
    pub id: PatId,
    pub title: Option<String>,
    pub user_id: UserId,
}

pub struct UpdatePatHandler<PR: PatRepository> {
    pat_repository: PR,
}

impl<PR: PatRepository> UpdatePatHandler<PR> {
    pub fn new(pat_repository: PR) -> Self {
        Self { pat_repository }
    }
}

#[async_trait::async_trait]
impl<PR: PatRepository> Handler<UpdatePatCommand> for UpdatePatHandler<PR> {
    type Response = PersonalAccessToken;
    type Error = UpdatePatError;

    async fn handle(&self, cmd: UpdatePatCommand) -> Result<Self::Response, Self::Error> {
        let mut pat = self
            .pat_repository
            .find_by_id(cmd.id, cmd.user_id)
            .await?
            .ok_or(PatError::NotFound(cmd.id.as_inner()))?;

        if let Some(title) = cmd.title {
            let title = PatTitle::new(title)?;

            pat.set_title(title);
        }

        self.pat_repository.save(&pat).await?;

        Ok(pat)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdatePatError {
    #[error(transparent)]
    Pat(#[from] PatError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
