use uuid::Uuid;

use super::{Account, Error};
use crate::common::{Creatable, Findable};

pub trait AccountRepository:
    Findable<Params = AccountFindParams, Output = Result<Account, Error>>
    + Creatable<Data = AccountCreateData, Output = Result<Uuid, Error>>
    + Send
    + Sync
    + 'static
{
}

#[derive(Debug, Clone, Default)]
pub struct AccountFindParams {
    pub provider_id: String,
    pub account_id: String,
}

#[derive(Debug, Clone, Default)]
pub struct AccountCreateData {
    pub email: String,
    pub display_name: Option<String>,
    pub provider_id: String,
    pub account_id: String,
    pub password_hash: Option<String>,
}
