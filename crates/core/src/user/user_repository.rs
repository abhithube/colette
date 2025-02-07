use uuid::Uuid;

use super::{Error, User};
use crate::common::{Creatable, Findable};

pub trait UserRepository:
    Findable<Params = UserFindParams, Output = Result<User, Error>>
    + Creatable<Data = UserCreateData, Output = Result<Uuid, Error>>
    + Send
    + Sync
    + 'static
{
}

#[derive(Debug, Clone)]
pub enum UserFindParams {
    Id(Uuid),
    Email(String),
}

#[derive(Debug, Clone)]
pub struct UserCreateData {
    pub email: String,
    pub password: String,
}
