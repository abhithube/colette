use std::sync::Arc;

use axum::extract::FromRef;
use colette_core::{auth::AuthService, feeds::FeedsService, profiles::ProfilesService};

pub const SESSION_KEY: &str = "session";

#[derive(Clone, FromRef)]
pub struct Context {
    pub auth_service: Arc<AuthService>,
    pub profiles_service: Arc<ProfilesService>,
    pub feeds_service: Arc<FeedsService>,
}
