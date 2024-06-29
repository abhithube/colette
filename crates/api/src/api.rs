use std::sync::Arc;

use axum::extract::FromRef;
use colette_core::auth::AuthService;

#[derive(Clone, FromRef)]
pub struct Context {
    pub auth_service: Arc<AuthService>,
}
