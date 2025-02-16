use chrono::{DateTime, Utc};
use colette_util::{api_key, password};
use uuid::Uuid;

use super::{
    ApiKey, ApiKeySearchParams, Error,
    api_key_repository::{ApiKeyCreateData, ApiKeyFindParams, ApiKeyRepository, ApiKeyUpdateData},
};
use crate::{
    auth,
    common::{IdParams, Paginated},
};

pub struct ApiKeyService {
    repository: Box<dyn ApiKeyRepository>,
}

impl ApiKeyService {
    pub fn new(repository: impl ApiKeyRepository) -> Self {
        Self {
            repository: Box::new(repository),
        }
    }

    pub async fn list_api_keys(&self, user_id: Uuid) -> Result<Paginated<ApiKey>, Error> {
        let api_keys = self
            .repository
            .find(ApiKeyFindParams {
                user_id,
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: api_keys,
            ..Default::default()
        })
    }

    pub async fn validate_api_key(&self, value: String) -> Result<Uuid, Error> {
        let lookup_hash = api_key::hash(&value);

        let api_key = self
            .repository
            .search(ApiKeySearchParams { lookup_hash })
            .await?;

        if let Some(api_key) = api_key {
            let valid = password::verify(&value, &api_key.verification_hash)?;
            if valid {
                return Ok(api_key.user_id);
            }
        }

        Err(Error::Auth(auth::Error::NotAuthenticated))
    }

    pub async fn get_api_key(&self, id: Uuid, user_id: Uuid) -> Result<ApiKey, Error> {
        let mut api_keys = self
            .repository
            .find(ApiKeyFindParams {
                id: Some(id),
                user_id,
                ..Default::default()
            })
            .await?;
        if api_keys.is_empty() {
            return Err(Error::NotFound(id));
        }

        Ok(api_keys.swap_remove(0))
    }

    pub async fn create_api_key(
        &self,
        data: ApiKeyCreate,
        user_id: Uuid,
    ) -> Result<ApiKeyCreated, Error> {
        let value = api_key::generate();

        let id = self
            .repository
            .create(ApiKeyCreateData {
                lookup_hash: api_key::hash(&value),
                verification_hash: password::hash(&value)?,
                title: data.title,
                preview: format!(
                    "{}...{}",
                    &value[0..8],
                    &value[value.len() - 4..value.len()]
                ),
                user_id,
            })
            .await?;

        let api_key = self.get_api_key(id, user_id).await?;

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
        self.repository
            .update(IdParams::new(id, user_id), data.into())
            .await?;

        self.get_api_key(id, user_id).await
    }

    pub async fn delete_api_key(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        self.repository.delete(IdParams::new(id, user_id)).await
    }
}

impl From<ApiKeyUpdate> for ApiKeyUpdateData {
    fn from(value: ApiKeyUpdate) -> Self {
        Self { title: value.title }
    }
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

#[derive(Debug, Clone, Default)]
pub struct ApiKeySearched {
    pub verification_hash: String,
    pub user_id: Uuid,
}
