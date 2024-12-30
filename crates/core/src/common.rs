use std::vec::IntoIter;

use uuid::Uuid;

pub const PAGINATION_LIMIT: u64 = 24;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct NonEmptyString(String);

impl TryFrom<String> for NonEmptyString {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ValidationError::Empty);
        }

        Ok(NonEmptyString(value))
    }
}

impl From<NonEmptyString> for String {
    fn from(value: NonEmptyString) -> Self {
        value.0
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "Vec<T>", into = "Vec<T>")]
pub struct NonEmptyVec<T: Clone>(Vec<T>);

impl<T: Clone> TryFrom<Vec<T>> for NonEmptyVec<T> {
    type Error = ValidationError;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ValidationError::Empty);
        }

        Ok(NonEmptyVec(value))
    }
}

impl<T: Clone> From<NonEmptyVec<T>> for Vec<T> {
    fn from(value: NonEmptyVec<T>) -> Self {
        value.0
    }
}

impl<T: Clone> IntoIterator for NonEmptyVec<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: Clone> FromIterator<T> for NonEmptyVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        NonEmptyVec(Vec::from_iter(iter))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("cannot be empty")]
    Empty,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub cursor: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct IdParams {
    pub id: Uuid,
    pub user_id: Uuid,
}

impl IdParams {
    pub fn new(id: Uuid, user_id: Uuid) -> Self {
        Self { id, user_id }
    }
}

#[async_trait::async_trait]
pub trait Findable {
    type Params;
    type Output;

    async fn find(&self, params: Self::Params) -> Self::Output;
}

#[async_trait::async_trait]
pub trait Creatable {
    type Data;
    type Output;

    async fn create(&self, data: Self::Data) -> Self::Output;
}

#[async_trait::async_trait]
pub trait Updatable {
    type Params;
    type Data;
    type Output;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output;
}

#[async_trait::async_trait]
pub trait Deletable {
    type Params;
    type Output;

    async fn delete(&self, params: Self::Params) -> Self::Output;
}
