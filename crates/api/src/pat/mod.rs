use axum::{Router, routing};
use chrono::{DateTime, Utc};
use colette_handler::PersonalAccessTokenDto;
use uuid::Uuid;

use crate::{ApiState, pagination::Paginated};

mod create_pat;
mod delete_pat;
mod get_pat;
mod list_pats;
mod update_pat;

const PERSONAL_ACCESS_TOKENS_TAG: &str = "Personal Access Tokens";

#[derive(utoipa::OpenApi)]
#[openapi(
    components(schemas(
        PersonalAccessToken, Paginated<PersonalAccessToken>, create_pat::PatCreate, create_pat::PatCreated, update_pat::PatUpdate
    )),
    paths(
        list_pats::handler, create_pat::handler, get_pat::handler, update_pat::handler, delete_pat::handler
    )
)]
pub(crate) struct PersonalAccessTokensApi;

impl PersonalAccessTokensApi {
    pub(crate) fn router() -> Router<ApiState> {
        Router::new()
            .route("/pats", routing::get(list_pats::handler))
            .route("/pats", routing::post(create_pat::handler))
            .route("/pats/{id}", routing::get(get_pat::handler))
            .route("/pats/{id}", routing::patch(update_pat::handler))
            .route("/pats/{id}", routing::delete(delete_pat::handler))
    }
}

/// PAT, used for long-lived token access to the API
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PersonalAccessToken {
    id: Uuid,
    title: String,
    preview: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<PersonalAccessTokenDto> for PersonalAccessToken {
    fn from(value: PersonalAccessTokenDto) -> Self {
        Self {
            id: value.id,
            title: value.title,
            preview: value.preview,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
