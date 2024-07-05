use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use colette_core::{auth, common::Session};
use serde::{Deserialize, Serialize};

use crate::{api::SESSION_KEY, error::Error};

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionDto {
    pub user_id: String,
    pub profile_id: String,
}

impl<'a> From<&'a SessionDto> for Session<'a> {
    fn from(value: &'a SessionDto) -> Self {
        Self {
            user_id: value.user_id.as_str(),
            profile_id: value.profile_id.as_str(),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for SessionDto
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session_store = tower_sessions::Session::from_request_parts(req, state)
            .await
            .map_err(|_| Error::Auth(auth::Error::NotAuthenticated))?;

        let session = session_store
            .get::<SessionDto>(SESSION_KEY)
            .await
            .map_err(|_| Error::Auth(auth::Error::NotAuthenticated))?
            .ok_or(Error::Auth(auth::Error::NotAuthenticated))?;

        Ok(session)
    }
}
