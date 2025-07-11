use std::sync::Arc;

use chrono::{DateTime, Utc};
use colette_util::{
    argon2_hash, argon2_verify, base64_encode, hex_encode, random_generate, sha256_hash,
};
use uuid::Uuid;

use super::{ApiKey, ApiKeyCursor, ApiKeyParams, ApiKeyRepository, Error};
use crate::common::{PAGINATION_LIMIT, Paginated, Paginator};

pub struct ApiKeyService {
    repository: Arc<dyn ApiKeyRepository>,
}

impl ApiKeyService {
    pub fn new(repository: Arc<dyn ApiKeyRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_api_keys(
        &self,
        query: ApiKeyListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<ApiKey>, Error> {
        let cursor = query
            .cursor
            .map(|e| Paginator::decode_cursor::<ApiKeyCursor>(&e))
            .transpose()?;

        let api_keys = self
            .repository
            .query(ApiKeyParams {
                user_id: Some(user_id),
                cursor: cursor.map(|e| e.created_at),
                limit: Some(PAGINATION_LIMIT + 1),
                ..Default::default()
            })
            .await?;

        let data = Paginator::paginate(api_keys, PAGINATION_LIMIT)?;

        Ok(data)
    }

    pub async fn validate_api_key(&self, value: String) -> Result<ApiKey, Error> {
        let lookup_hash = hex_encode(&sha256_hash(&value));

        let Some(api_key) = self.repository.find_by_lookup_hash(lookup_hash).await? else {
            return Err(Error::Auth);
        };

        let valid = argon2_verify(&value, &api_key.verification_hash)?;
        if !valid {
            return Err(Error::Auth);
        }

        Ok(api_key)
    }

    pub async fn get_api_key(&self, id: Uuid, user_id: Uuid) -> Result<ApiKey, Error> {
        let mut api_keys = self
            .repository
            .query(ApiKeyParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if api_keys.is_empty() {
            return Err(Error::NotFound(id));
        }

        let api_key = api_keys.swap_remove(0);
        if api_key.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        Ok(api_key)
    }

    pub async fn create_api_key(
        &self,
        data: ApiKeyCreate,
        user_id: Uuid,
    ) -> Result<ApiKeyCreated, Error> {
        let value = base64_encode(&random_generate(32));

        let lookup_hash = hex_encode(&sha256_hash(&value));
        let verification_hash = argon2_hash(&value)?;

        let api_key = ApiKey::builder()
            .lookup_hash(lookup_hash)
            .verification_hash(verification_hash)
            .title(data.title)
            .preview(format!(
                "{}...{}",
                &value[0..8],
                &value[value.len() - 4..value.len()]
            ))
            .user_id(user_id)
            .build();

        self.repository.save(&api_key).await?;

        Ok(ApiKeyCreated {
            id: api_key.id,
            title: api_key.title,
            value,
            created_at: api_key.created_at,
        })
    }

    pub async fn update_api_key(
        &self,
        id: Uuid,
        data: ApiKeyUpdate,
        user_id: Uuid,
    ) -> Result<ApiKey, Error> {
        let Some(mut api_key) = self.repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if api_key.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        if let Some(title) = data.title {
            api_key.title = title;
        }

        api_key.updated_at = Utc::now();
        self.repository.save(&api_key).await?;

        Ok(api_key)
    }

    pub async fn delete_api_key(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        let Some(api_key) = self.repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if api_key.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.repository.delete_by_id(id).await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyListQuery {
    pub cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApiKeyCreate {
    pub title: String,
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyCreated {
    pub id: Uuid,
    pub title: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyUpdate {
    pub title: Option<String>,
}
