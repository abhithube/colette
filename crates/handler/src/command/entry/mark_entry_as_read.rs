use colette_authentication::UserId;
use colette_common::RepositoryError;
use colette_crud::{EntryError, EntryId, EntryRepository};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct MarkEntryAsReadCommand {
    pub id: EntryId,
    pub user_id: UserId,
}

pub struct MarkEntryAsReadHandler<ER: EntryRepository> {
    entry_repository: ER,
}

impl<ER: EntryRepository> MarkEntryAsReadHandler<ER> {
    pub fn new(entry_repository: ER) -> Self {
        Self { entry_repository }
    }
}

impl<ER: EntryRepository> Handler<MarkEntryAsReadCommand> for MarkEntryAsReadHandler<ER> {
    type Response = ();
    type Error = MarkEntryAsReadError;

    async fn handle(&self, cmd: MarkEntryAsReadCommand) -> Result<Self::Response, Self::Error> {
        let mut entry = self
            .entry_repository
            .find_by_id(cmd.id, cmd.user_id)
            .await?
            .ok_or(EntryError::NotFound(cmd.id.as_inner()))?;

        entry.mark_as_read()?;

        self.entry_repository.save(&entry).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MarkEntryAsReadError {
    #[error(transparent)]
    Entry(#[from] EntryError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
