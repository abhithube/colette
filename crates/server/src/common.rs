use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::FromRef,
    response::{IntoResponse, Response},
    Json,
};
use colette_core::{
    auth::AuthService, collections::CollectionsService, common, entries::EntriesService,
    feeds::FeedsService, profiles::ProfilesService,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{collections::Collection, entries::Entry, feeds::Feed, profiles::Profile};

#[derive(Clone, FromRef)]
pub struct Context {
    pub auth_service: Arc<AuthService>,
    pub collections_service: Arc<CollectionsService>,
    pub entries_service: Arc<EntriesService>,
    pub feeds_service: Arc<FeedsService>,
    pub profiles_service: Arc<ProfilesService>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(names("id"))]
pub struct Id(pub String);

#[derive(Debug, Serialize, ToSchema)]
#[aliases(CollectionList = Paginated<Collection>, EntryList = Paginated<Entry>, FeedList = Paginated<Feed>, ProfileList = Paginated<Profile>)]
#[serde(rename_all = "camelCase")]
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
pub struct BaseError {
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub params: HashMap<String, String>,
}

impl IntoResponse for BaseError {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}
