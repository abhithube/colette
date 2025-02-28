use chrono::{DateTime, Utc};
use colette_core::{Account, ApiKey, Collection, Stream, User, api_key::ApiKeySearched};
pub use entity::*;

mod entity;

fn parse_timestamp(value: i32) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp(value.into(), 0)
}

pub struct AccountWithUser {
    pub account: accounts::Model,
    pub user: users::Model,
}

impl From<AccountWithUser> for Account {
    fn from(value: AccountWithUser) -> Self {
        Self {
            id: value.user.id.parse().unwrap(),
            email: value.user.email,
            provider_id: value.account.provider_id,
            account_id: value.account.account_id,
            password_hash: value.account.password_hash,
        }
    }
}

impl From<api_keys::Model> for ApiKey {
    fn from(value: api_keys::Model) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            title: value.title,
            preview: value.preview,
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.created_at),
        }
    }
}

impl From<api_keys::Model> for ApiKeySearched {
    fn from(value: api_keys::Model) -> Self {
        Self {
            verification_hash: value.verification_hash,
            user_id: value.user_id.parse().unwrap(),
        }
    }
}

impl From<collections::Model> for Collection {
    fn from(value: collections::Model) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            title: value.title,
            filter: serde_json::from_str(&value.filter_raw).unwrap(),
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
        }
    }
}

impl From<streams::Model> for Stream {
    fn from(value: streams::Model) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            title: value.title,
            filter: serde_json::from_str(&value.filter_raw).unwrap(),
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
        }
    }
}

impl From<users::Model> for User {
    fn from(value: users::Model) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            email: value.email,
            display_name: value.display_name,
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
        }
    }
}
