use uuid::Uuid;

use super::Backup;
use crate::RepositoryError;

#[async_trait::async_trait]
pub trait BackupRepository: Send + Sync + 'static {
    async fn import(&self, params: ImportBackupParams) -> Result<(), RepositoryError>;
}

pub struct ImportBackupParams {
    pub backup: Backup,
    pub user_id: Uuid,
}
