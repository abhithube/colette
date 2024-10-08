use uuid::Uuid;

pub const PAGINATION_LIMIT: u64 = 24;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct NonEmptyString(String);

impl TryFrom<String> for NonEmptyString {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ValidationError::EmptyString);
        }

        Ok(NonEmptyString(value))
    }
}

impl From<NonEmptyString> for String {
    fn from(value: NonEmptyString) -> Self {
        value.0
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TagsLinkAction {
    Add,
    Set,
    Remove,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagsLink {
    pub data: Vec<NonEmptyString>,
    pub action: TagsLinkAction,
}

#[derive(Clone, Debug)]
pub struct TagsLinkData {
    pub data: Vec<String>,
    pub action: TagsLinkAction,
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
