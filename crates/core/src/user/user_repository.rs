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

#[derive(Clone, Debug)]
pub enum UserFindParams {
    Id(Uuid),
    Email(String),
}

#[derive(Clone, Debug)]
pub struct UserCreateData {
    pub email: String,
    pub password: String,
}
