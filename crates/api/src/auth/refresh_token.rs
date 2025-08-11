use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use colette_core::{Handler as _, auth::RefreshAccessTokenCommand};

use crate::{
    ApiState,
    auth::{AUTH_TAG, REFRESH_COOKIE, TokenData},
    common::{ApiError, build_cookie},
};

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
        .refresh_access_token
        .handle(RefreshAccessTokenCommand {
            refresh_token: refresh_cookie.value().to_string(),
        })
        .await
    {
        Ok(tokens) => {
            let refresh_cookie = build_cookie(
                (REFRESH_COOKIE, tokens.refresh_token.clone()),
                Some(tokens.refresh_expires_in.num_seconds()),
            );

            Ok((jar.add(refresh_cookie), OkResponse(tokens.into())))
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
