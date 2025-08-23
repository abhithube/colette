use crate::{
    Handler,
    auth::{PatError, PatId, PatTitle, PersonalAccessToken, UserError, UserId, UserRepository},
    common::RepositoryError,
};

#[derive(Debug, Clone)]
pub struct UpdatePatCommand {
    pub id: PatId,
    pub title: Option<String>,
    pub user_id: UserId,
}

pub struct UpdatePatHandler<UR: UserRepository> {
    user_repository: UR,
}

impl<UR: UserRepository> UpdatePatHandler<UR> {
    pub fn new(user_repository: UR) -> Self {
        Self { user_repository }
    }
}

#[async_trait::async_trait]
impl<UR: UserRepository> Handler<UpdatePatCommand> for UpdatePatHandler<UR> {
    type Response = PersonalAccessToken;
    type Error = UpdatePatError;

    async fn handle(&self, cmd: UpdatePatCommand) -> Result<Self::Response, Self::Error> {
        let mut user = self
            .user_repository
            .find_by_id(cmd.user_id)
            .await?
            .ok_or(UpdatePatError::NotAuthenticated)?;

        let pat = user
            .get_personal_access_token(cmd.id)
            .ok_or(UserError::Pat(PatError::NotFound(cmd.id)))?;

        if let Some(title) = cmd.title {
            let title = PatTitle::new(title).map_err(UserError::Pat)?;

            pat.set_title(title);
        }

        let data = pat.to_owned();

        self.user_repository.save(&user).await?;

        Ok(data)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdatePatError {
    #[error("user not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    User(#[from] UserError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
