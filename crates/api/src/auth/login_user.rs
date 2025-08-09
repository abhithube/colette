use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use colette_core::{
    Handler as _,
    auth::{LoginUserCommand, LoginUserError},
};
use email_address::EmailAddress;

use super::{AUTH_TAG, REFRESH_COOKIE, TokenData};
use crate::{
    ApiState,
    common::{ApiError, Json, NonEmptyString, build_cookie},
};

#[utoipa::path(
  post,
  path = "/login",
  request_body = LoginPayload,
  responses(OkResponse, ErrResponse),
  operation_id = "loginUser",
  description = "Login to a user account",
  tag = AUTH_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    jar: CookieJar,
    Json(body): Json<LoginPayload>,
) -> Result<impl IntoResponse, ErrResponse> {
    match state
        .login_user
        .handle(LoginUserCommand {
            email: body.email.to_string(),
            password: body.password.into(),
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
        Err(e) => match e {
            LoginUserError::NotAuthenticated => Err(ErrResponse::Unauthorized(e.into())),
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct LoginPayload {
    #[schema(value_type = String, format = "email")]
    email: EmailAddress,
    #[schema(value_type = String, min_length = 1)]
    password: NonEmptyString,
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
    #[response(status = StatusCode::UNAUTHORIZED, description = "Bad credentials")]
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
                (StatusCode::UNAUTHORIZED, ApiError::bad_credentials()).into_response()
            }
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
