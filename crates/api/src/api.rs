use std::sync::Arc;

use axum::extract::FromRef;
use colette_core::{auth::AuthService, feeds::FeedsService, profiles::ProfilesService};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{feeds::Feed, profiles::Profile};

pub const SESSION_KEY: &str = "session";

#[derive(Clone, FromRef)]
pub struct Context {
    pub auth_service: Arc<AuthService>,
    pub profiles_service: Arc<ProfilesService>,
    pub feeds_service: Arc<FeedsService>,
}

#[derive(Debug, Serialize, ToSchema)]
#[aliases(FeedList = Paginated<Feed>, ProfileList = Paginated<Profile>)]
pub struct Paginated<T: Serialize> {
    has_more: bool,
    data: Vec<T>,
}

impl<T> From<colette_core::common::Paginated<T>> for Paginated<T>
where
    T: Serialize,
{
    fn from(value: colette_core::common::Paginated<T>) -> Self {
        Self {
            has_more: value.has_more,
            data: value.data,
        }
    }
}
