use chrono::{DateTime, Utc};
use colette_core::feeds;
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    pub id: String,
    #[schema(format = "uri")]
    pub link: String,
    pub title: String,
    #[schema(format = "uri")]
    pub url: Option<String>,
    pub custom_title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub unread_count: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateFeed {
    pub url: Url,
}

impl From<colette_core::Feed> for Feed {
    fn from(value: colette_core::Feed) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            url: value.url,
            custom_title: value.custom_title,
            created_at: value.created_at,
            updated_at: value.updated_at,
            unread_count: value.unread_count,
        }
    }
}

impl<'a> From<&'a CreateFeed> for feeds::CreateFeed<'a> {
    fn from(value: &'a CreateFeed) -> Self {
        Self {
            url: value.url.as_str(),
        }
    }
}
