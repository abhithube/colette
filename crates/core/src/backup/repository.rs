use colette_authentication::UserId;
use colette_common::RepositoryError;

use crate::backup::Backup;

#[async_trait::async_trait]
pub trait BackupRepository: Sync {
    async fn import(&self, params: ImportBackupParams) -> Result<(), RepositoryError>;
}

pub struct ImportBackupParams {
    pub backup: Backup,
    pub user_id: UserId,
}
