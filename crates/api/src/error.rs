use axum::{
    extract::rejection::{JsonRejection, QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::{auth, profiles, users};
use thiserror::Error;
use tower_sessions::session;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    QueryRejection(#[from] QueryRejection),

    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),

    #[error(transparent)]
    Session(#[from] session::Error),

    #[error(transparent)]
    Auth(#[from] auth::Error),

    #[error(transparent)]
    Profiles(#[from] profiles::Error),

    #[error(transparent)]
    Users(#[from] users::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::QueryRejection(e) => (StatusCode::BAD_REQUEST, e).into_response(),
            Error::JsonRejection(e) => (StatusCode::BAD_REQUEST, e).into_response(),
            Error::Auth(auth::Error::NotAuthenticated) => {
                (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
            }
            Error::Users(users::Error::NotFound(_))
            | Error::Profiles(profiles::Error::NotFound(_)) => {
                (StatusCode::NOT_FOUND, self.to_string()).into_response()
            }
            Error::Auth(auth::Error::Users(e)) => match e {
                users::Error::Conflict(_) => (StatusCode::CONFLICT, e.to_string()).into_response(),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            },
            Error::Profiles(profiles::Error::DeletingDefault) => {
                (StatusCode::CONFLICT, self.to_string()).into_response()
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response(),
        }
    }
}
