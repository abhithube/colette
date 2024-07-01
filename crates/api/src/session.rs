use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use colette_core::{auth, common::Session};
use tower_sessions::Session as TSession;

use crate::error::Error;

pub struct SessionAuth(pub Session);

#[async_trait]
impl<S> FromRequestParts<S> for SessionAuth
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = TSession::from_request_parts(req, state)
            .await
            .map_err(|_| Error::Auth(auth::Error::NotAuthenticated))?;

        let user = session
            .get::<Session>("session")
            .await
            .map_err(|_| Error::Auth(auth::Error::NotAuthenticated))?
            .ok_or(Error::Auth(auth::Error::NotAuthenticated))?;

        Ok(SessionAuth(user))
    }
}
