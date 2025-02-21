use uuid::Uuid;

use super::{Error, User};
use crate::common::Findable;

pub trait UserRepository:
    Findable<Params = UserFindParams, Output = Result<User, Error>> + Send + Sync + 'static
{
}

#[derive(Debug, Clone)]
pub struct UserFindParams {
    pub id: Uuid,
}
