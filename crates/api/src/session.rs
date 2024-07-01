use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use colette_core::{auth, common::Session};

use crate::{api::SESSION_KEY, error::Error};

pub struct SessionAuth(pub Session);

#[async_trait]
impl<S> FromRequestParts<S> for SessionAuth
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = tower_sessions::Session::from_request_parts(req, state)
            .await
            .map_err(|_| Error::Auth(auth::Error::NotAuthenticated))?;

        let user = session
            .get::<Session>(SESSION_KEY)
            .await
            .map_err(|_| Error::Auth(auth::Error::NotAuthenticated))?
            .ok_or(Error::Auth(auth::Error::NotAuthenticated))?;

        Ok(SessionAuth(user))
    }
}
