use std::sync::Arc;

use axum::{
    extract::FromRef,
    response::{IntoResponse, Response},
    Json,
};
use colette_core::{auth::AuthService, common, feeds::FeedsService, profiles::ProfilesService};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{feeds::Feed, profiles::Profile};

#[derive(Clone, FromRef)]
pub struct Context {
    pub auth_service: Arc<AuthService>,
    pub profiles_service: Arc<ProfilesService>,
    pub feeds_service: Arc<FeedsService>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(names("id"))]
pub struct Id(pub String);

#[derive(Debug, Serialize, ToSchema)]
#[aliases(FeedList = Paginated<Feed>, ProfileList = Paginated<Profile>)]
pub struct Paginated<T: Serialize> {
    pub has_more: bool,
    pub data: Vec<T>,
}

impl<T, U> From<common::Paginated<U>> for Paginated<T>
where
    T: From<U> + Serialize,
{
    fn from(value: common::Paginated<U>) -> Self {
        Self {
            has_more: value.has_more,
            data: value.data.into_iter().map(T::from).collect(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Error {
    pub message: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}