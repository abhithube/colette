use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

pub const PAGINATION_LIMIT: u64 = 24;

#[derive(Clone, Debug, Default)]
pub struct NonEmptyString(String);

impl From<NonEmptyString> for String {
    fn from(value: NonEmptyString) -> Self {
        value.0
    }
}

impl TryFrom<String> for NonEmptyString {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ValidationError::EmptyString);
        }

        Ok(NonEmptyString(value))
    }
}

impl Serialize for NonEmptyString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for NonEmptyString {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;

        NonEmptyString::try_from(s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("cannot be empty")]
    EmptyString,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub cursor: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct IdParams {
    pub id: Uuid,
    pub profile_id: Uuid,
}

impl IdParams {
    pub fn new(id: Uuid, profile_id: Uuid) -> Self {
        Self { id, profile_id }
    }
}

#[async_trait::async_trait]
pub trait Findable: Send + Sync {
    type Params;
    type Output;

    async fn find(&self, params: Self::Params) -> Self::Output;
}

#[async_trait::async_trait]
pub trait Creatable: Send + Sync {
    type Data;
    type Output;

    async fn create(&self, data: Self::Data) -> Self::Output;
}

#[async_trait::async_trait]
pub trait Updatable: Send + Sync {
    type Params;
    type Data;
    type Output;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output;
}

#[async_trait::async_trait]
pub trait Deletable: Send + Sync {
    type Params;
    type Output;

    async fn delete(&self, params: Self::Params) -> Self::Output;
}
