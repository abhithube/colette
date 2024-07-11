use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use axum_valid::Valid;
use colette_core::entries::EntriesService;

use super::{model::ListEntriesQuery, Entry};
use crate::{common::Paginated, entries::model::ListResponse, error::Error, session::Session};

#[utoipa::path(
    get,
    path = "",
    params(ListEntriesQuery),
    responses(ListResponse),
    operation_id = "listEntries",
    description = "List feed entries",
    tag = "Entries"
)]
#[axum::debug_handler]
pub async fn list_entries(
    State(service): State<Arc<EntriesService>>,
    Valid(Query(query)): Valid<Query<ListEntriesQuery>>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = service
        .list(query.into(), session.into())
        .await
        .map(Paginated::<Entry>::from);

    match result {
        Ok(data) => Ok(ListResponse::Ok(data)),
        _ => Err(Error::Unknown),
    }
}
