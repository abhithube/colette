use chrono::{DateTime, Utc};
use colette_core::profiles;
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub title: String,
    #[schema(format = "uri")]
    pub image_url: Option<String>,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateProfile {
    #[schema(min_length = 1)]
    pub title: String,
    #[schema(nullable = false)]
    pub image_url: Option<Url>,
}

impl From<colette_core::Profile> for Profile {
    fn from(value: colette_core::Profile) -> Self {
        Self {
            id: value.id,
            title: value.title,
            image_url: value.image_url,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl<'a> From<&'a CreateProfile> for profiles::CreateProfile<'a> {
    fn from(value: &'a CreateProfile) -> Self {
        Self {
            title: value.title.as_str(),
            image_url: value.image_url.as_ref().map(|e| e.as_str()),
        }
    }
}
