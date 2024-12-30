use dyn_clone::DynClone;
use uuid::Uuid;

use crate::common::{Creatable, Findable};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
}

pub trait UserRepository:
    Findable<Params = UserFindParams, Output = Result<User, Error>>
    + Creatable<Data = UserCreateData, Output = Result<Uuid, Error>>
    + Send
    + Sync
    + DynClone
    + 'static
{
}

dyn_clone::clone_trait_object!(UserRepository);

#[derive(Clone, Debug)]
pub enum UserFindParams {
    Id(Uuid),
    Email(String),
}

#[derive(Clone, Debug, Default)]
pub struct UserCreateData {
    pub email: String,
    pub password: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    NotFound(#[from] NotFoundError),

    #[error("user already exists with email: {0}")]
    Conflict(String),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum NotFoundError {
    #[error("user not found with id: {0}")]
    Id(Uuid),

    #[error("user not found with email: {0}")]
    Email(String),
}
