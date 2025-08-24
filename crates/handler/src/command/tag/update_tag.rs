use colette_authentication::UserId;
use colette_common::RepositoryError;
use colette_core::tag::{TagError, TagId, TagRepository, TagTitle};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct UpdateTagCommand {
    pub id: TagId,
    pub title: Option<String>,
    pub user_id: UserId,
}

pub struct UpdateTagHandler<TR: TagRepository> {
    tag_repository: TR,
}

impl<TR: TagRepository> UpdateTagHandler<TR> {
    pub fn new(tag_repository: TR) -> Self {
        Self { tag_repository }
    }
}

#[async_trait::async_trait]
impl<TR: TagRepository> Handler<UpdateTagCommand> for UpdateTagHandler<TR> {
    type Response = ();
    type Error = UpdateTagError;

    async fn handle(&self, cmd: UpdateTagCommand) -> Result<Self::Response, Self::Error> {
        let mut tag = self
            .tag_repository
            .find_by_id(cmd.id, cmd.user_id)
            .await?
            .ok_or(TagError::NotFound(cmd.id.as_inner()))?;

        let title = cmd.title.clone().map(TagTitle::new).transpose()?;

        if let Some(title) = title {
            tag.set_title(title);
        }

        self.tag_repository.save(&tag).await.map_err(|e| match e {
            RepositoryError::Duplicate => {
                UpdateTagError::Tag(TagError::Conflict(cmd.title.unwrap()))
            }
            _ => UpdateTagError::Repository(e),
        })?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateTagError {
    #[error(transparent)]
    Tag(#[from] TagError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
