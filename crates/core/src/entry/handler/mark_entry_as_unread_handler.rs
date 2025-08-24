use crate::{
    Handler,
    auth::UserId,
    common::RepositoryError,
    entry::{EntryError, EntryId, EntryRepository},
};

#[derive(Debug, Clone)]
pub struct MarkEntryAsUnreadCommand {
    pub id: EntryId,
    pub user_id: UserId,
}

pub struct MarkEntryAsUnreadHandler<ER: EntryRepository> {
    entry_repository: ER,
}

impl<ER: EntryRepository> MarkEntryAsUnreadHandler<ER> {
    pub fn new(entry_repository: ER) -> Self {
        Self { entry_repository }
    }
}

#[async_trait::async_trait]
impl<ER: EntryRepository> Handler<MarkEntryAsUnreadCommand> for MarkEntryAsUnreadHandler<ER> {
    type Response = ();
    type Error = MarkEntryAsUnreadError;

    async fn handle(&self, cmd: MarkEntryAsUnreadCommand) -> Result<Self::Response, Self::Error> {
        let mut entry = self
            .entry_repository
            .find_by_id(cmd.id, cmd.user_id)
            .await?
            .ok_or_else(|| MarkEntryAsUnreadError::Entry(EntryError::NotFound(cmd.id)))?;

        entry.mark_as_unread()?;

        self.entry_repository.save(&entry).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MarkEntryAsUnreadError {
    #[error(transparent)]
    Entry(#[from] EntryError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
