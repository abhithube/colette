use chrono::{DateTime, Utc};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{ApiState, bookmark::BookmarkFilter};
use crate::common::Paginated;

mod create_collection;
mod delete_collection;
mod get_collection;
mod list_collections;
mod update_collection;

pub const COLLECTIONS_TAG: &str = "Collections";

#[derive(OpenApi)]
#[openapi(components(schemas(Collection, Paginated<Collection>, create_collection::CollectionCreate, update_collection::CollectionUpdate)))]
pub struct CollectionApi;

impl CollectionApi {
    pub fn router() -> OpenApiRouter<ApiState> {
        OpenApiRouter::with_openapi(CollectionApi::openapi())
            .routes(routes!(
                list_collections::handler,
                create_collection::handler
            ))
            .routes(routes!(
                get_collection::handler,
                update_collection::handler,
                delete_collection::handler
            ))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: Uuid,
    pub title: String,
    pub filter: BookmarkFilter,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<colette_core::Collection> for Collection {
    fn from(value: colette_core::Collection) -> Self {
        Self {
            id: value.id,
            title: value.title,
            filter: value.filter.into(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
