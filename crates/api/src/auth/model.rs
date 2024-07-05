use chrono::{DateTime, Utc};
use colette_core::auth;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    #[schema(format = "email")]
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct Register {
    #[schema(format = "email")]
    pub email: String,
    #[schema(min_length = 1)]
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct Login {
    #[schema(format = "email")]
    pub email: String,
    #[schema(min_length = 1)]
    pub password: String,
}

impl From<colette_core::User> for User {
    fn from(value: colette_core::User) -> Self {
        Self {
            id: value.id,
            email: value.email,
            password: value.password,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl<'a> From<&'a Register> for auth::Register<'a> {
    fn from(value: &'a Register) -> Self {
        Self {
            email: value.email.as_str(),
            password: value.password.as_str(),
        }
    }
}

impl<'a> From<&'a Login> for auth::Login<'a> {
    fn from(value: &'a Login) -> Self {
        Self {
            email: value.email.as_str(),
            password: value.password.as_str(),
        }
    }
}
