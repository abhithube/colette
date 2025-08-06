use uuid::Uuid;

use super::{Backup, Error};

#[async_trait::async_trait]
pub trait BackupRepository: Send + Sync + 'static {
    async fn import(&self, params: ImportBackupParams) -> Result<(), Error>;
}

pub struct ImportBackupParams {
    pub backup: Backup,
    pub user_id: Uuid,
}
