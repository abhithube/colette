use bytes::Bytes;
use colette_core::{
    auth::UserId,
    backup::{Backup, BackupRepository, ImportBackupParams},
    common::RepositoryError,
};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct ImportBackupCommand {
    pub raw: Bytes,
    pub user_id: UserId,
}

pub struct ImportBackupHandler<BR: BackupRepository> {
    backup_repository: BR,
}

impl<BR: BackupRepository> ImportBackupHandler<BR> {
    pub fn new(backup_repository: BR) -> Self {
        Self { backup_repository }
    }
}

#[async_trait::async_trait]
impl<BR: BackupRepository> Handler<ImportBackupCommand> for ImportBackupHandler<BR> {
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
