use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;

use super::{AUTH_TAG, REFRESH_COOKIE, TokenData, build_refresh_cookie};
use crate::{ApiState, common::ApiError};

#[utoipa::path(
  post,
  path = "/token",
  responses(OkResponse, ErrResponse),
  operation_id = "refreshToken",
  description = "Generate a new access token, and rotate the refresh token",
  tag = AUTH_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, ErrResponse> {
    let Some(refresh_cookie) = jar.get(REFRESH_COOKIE) else {
        return Err(ErrResponse::Unauthorized(ApiError::forbidden()));
    };

    match state
        .auth_service
        .refresh_access_token(refresh_cookie.value())
        .await
    {
        Ok(tokens) => {
            let cookie = build_refresh_cookie(&tokens.refresh_token, tokens.refresh_expires_in);

            Ok((
                [(header::SET_COOKIE, cookie.to_string())],
                OkResponse(tokens.into()),
            ))
        }
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Access token for autheticated user")]
pub(super) struct OkResponse(TokenData);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, axum::Json(self.0)).into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::UNPROCESSABLE_ENTITY, description = "Invalid input")]
    UnprocessableEntity(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Unauthorized(_) => {
                (StatusCode::UNAUTHORIZED, ApiError::not_authenticated()).into_response()
            }
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
