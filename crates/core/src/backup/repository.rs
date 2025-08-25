use colette_authentication::UserId;
use colette_common::RepositoryError;

use crate::backup::Backup;

pub trait BackupRepository: Sync {
    fn import(
        &self,
        params: ImportBackupParams,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;
}

pub struct ImportBackupParams {
    pub backup: Backup,
    pub user_id: UserId,
}
