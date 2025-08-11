use crate::{backup::Backup, common::RepositoryError, user::UserId};

#[async_trait::async_trait]
pub trait BackupRepository: Send + Sync + 'static {
    async fn import(&self, params: ImportBackupParams) -> Result<(), RepositoryError>;
}

pub struct ImportBackupParams {
    pub backup: Backup,
    pub user_id: UserId,
}
