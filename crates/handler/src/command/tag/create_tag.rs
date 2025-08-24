use colette_core::{
    Tag,
    auth::UserId,
    common::RepositoryError,
    tag::{TagError, TagRepository, TagTitle},
};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct CreateTagCommand {
    pub title: String,
    pub user_id: UserId,
}

pub struct CreateTagHandler<TR: TagRepository> {
    tag_repository: TR,
}

impl<TR: TagRepository> CreateTagHandler<TR> {
    pub fn new(tag_repository: TR) -> Self {
        Self { tag_repository }
    }
}

#[async_trait::async_trait]
impl<TR: TagRepository> Handler<CreateTagCommand> for CreateTagHandler<TR> {
    type Response = Tag;
    type Error = CreateTagError;

    async fn handle(&self, cmd: CreateTagCommand) -> Result<Self::Response, Self::Error> {
        let title = TagTitle::new(cmd.title)?;

        let tag = Tag::new(title.clone(), cmd.user_id);

        self.tag_repository.save(&tag).await.map_err(|e| match e {
            RepositoryError::Duplicate => CreateTagError::Tag(TagError::Conflict(title)),
            _ => CreateTagError::Repository(e),
        })?;

        Ok(tag)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateTagError {
    #[error(transparent)]
    Tag(#[from] TagError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
