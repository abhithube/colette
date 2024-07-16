use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use colette_core::{auth, common};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Error;

pub const SESSION_KEY: &str = "session";

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub user_id: Uuid,
    pub profile_id: Uuid,
}

impl From<Session> for common::Session {
    fn from(value: Session) -> Self {
        Self {
            user_id: value.user_id,
            profile_id: value.profile_id,
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Session
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session_store = tower_sessions::Session::from_request_parts(req, state)
            .await
            .map_err(|_| Error::Auth(auth::Error::NotAuthenticated))?;

        let session = session_store
            .get::<Session>(SESSION_KEY)
            .await
            .map_err(|_| Error::Auth(auth::Error::NotAuthenticated))?
            .ok_or(Error::Auth(auth::Error::NotAuthenticated))?;

        Ok(session)
    }
}
