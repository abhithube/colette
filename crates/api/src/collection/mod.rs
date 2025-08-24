use axum::{Router, routing};
use chrono::{DateTime, Utc};
use colette_handler::CollectionDto;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{ApiState, bookmark::BookmarkFilter, pagination::Paginated};

mod create_collection;
mod delete_collection;
mod get_collection;
mod list_collections;
mod update_collection;

const COLLECTIONS_TAG: &str = "Collections";

#[derive(OpenApi)]
#[openapi(
    components(schemas(Collection, Paginated<Collection>, create_collection::CollectionCreate, update_collection::CollectionUpdate)),
    paths(list_collections::handler, create_collection::handler, get_collection::handler, update_collection::handler, delete_collection::handler)
)]
pub(crate) struct CollectionApi;

impl CollectionApi {
    pub(crate) fn router() -> Router<ApiState> {
        Router::new()
            .route("/", routing::get(list_collections::handler))
            .route("/", routing::post(create_collection::handler))
            .route("/{id}", routing::get(get_collection::handler))
            .route("/{id}", routing::patch(update_collection::handler))
            .route("/{id}", routing::delete(delete_collection::handler))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct Collection {
    id: Uuid,
    title: String,
    filter: BookmarkFilter,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<CollectionDto> for Collection {
    fn from(value: CollectionDto) -> Self {
        Self {
            id: value.id,
            title: value.title,
            filter: value.filter.into(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
