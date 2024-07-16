use axum::{
    extract::rejection::{JsonRejection, QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use colette_core::auth;
use tower_sessions::session;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    QueryRejection(#[from] QueryRejection),

    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),

    #[error(transparent)]
    Session(#[from] session::Error),

    #[error(transparent)]
    Auth(#[from] auth::Error),

    #[error("Unknown error")]
    Unknown,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let e = BaseError {
            message: self.to_string(),
        };

        match self {
            Error::QueryRejection(_) => (StatusCode::BAD_REQUEST, e).into_response(),
            Error::JsonRejection(_) => (StatusCode::BAD_REQUEST, e).into_response(),
            Error::Auth(auth::Error::NotAuthenticated) => {
                (StatusCode::UNAUTHORIZED, e).into_response()
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        }
    }
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[schema(title = "Error")]
pub struct BaseError {
    pub message: String,
}

impl IntoResponse for BaseError {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}
