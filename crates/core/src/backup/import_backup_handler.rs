use bytes::Bytes;
use uuid::Uuid;

use super::{Backup, BackupRepository};
use crate::{Handler, RepositoryError, backup::ImportBackupParams};

#[derive(Debug, Clone)]
pub struct ImportBackupCommand {
    pub raw: Bytes,
    pub user_id: Uuid,
}

pub struct ImportBackupHandler {
    backup_repository: Box<dyn BackupRepository>,
}

impl ImportBackupHandler {
    pub fn new(backup_repository: impl BackupRepository) -> Self {
        Self {
            backup_repository: Box::new(backup_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<ImportBackupCommand> for ImportBackupHandler {
    type Response = ();
    type Error = ImportBackupError;

    async fn handle(&self, cmd: ImportBackupCommand) -> Result<Self::Response, Self::Error> {
        let backup = serde_json::from_slice::<Backup>(&cmd.raw)?;

        self.backup_repository
            .import(ImportBackupParams {
                backup,
                user_id: cmd.user_id,
            })
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ImportBackupError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
