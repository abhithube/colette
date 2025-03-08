use chrono::{DateTime, Utc};
use colette_util::{api_key, password};
use uuid::Uuid;

use super::{
    ApiKey, ApiKeyCreateParams, ApiKeyDeleteParams, ApiKeyFindByIdParams, ApiKeyFindParams,
    ApiKeyRepository, ApiKeySearchParams, ApiKeyUpdateParams, Error,
};
use crate::{
    auth,
    common::{Paginated, TransactionManager},
};

pub struct ApiKeyService {
    repository: Box<dyn ApiKeyRepository>,
    tx_manager: Box<dyn TransactionManager>,
}

impl ApiKeyService {
    pub fn new(repository: impl ApiKeyRepository, tx_manager: impl TransactionManager) -> Self {
        Self {
            repository: Box::new(repository),
            tx_manager: Box::new(tx_manager),
        }
    }

    pub async fn list_api_keys(&self, user_id: Uuid) -> Result<Paginated<ApiKey>, Error> {
        let api_keys = self
            .repository
            .find_api_keys(ApiKeyFindParams {
                user_id: Some(user_id),
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
            .search_api_key(ApiKeySearchParams { lookup_hash })
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
            .find_api_keys(ApiKeyFindParams {
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
        let id = Uuid::new_v4();
        let value = api_key::generate();

        self.repository
            .create_api_key(ApiKeyCreateParams {
                id,
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
            id,
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
        let tx = self.tx_manager.begin().await?;

        let api_key = self
            .repository
            .find_api_key_by_id(&*tx, ApiKeyFindByIdParams { id })
            .await?;
        if api_key.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.repository
            .update_api_key(
                &*tx,
                ApiKeyUpdateParams {
                    id,
                    title: data.title,
                },
            )
            .await?;

        tx.commit().await?;

        self.get_api_key(id, user_id).await
    }

    pub async fn delete_api_key(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        let tx = self.tx_manager.begin().await?;

        let api_key = self
            .repository
            .find_api_key_by_id(&*tx, ApiKeyFindByIdParams { id })
            .await?;
        if api_key.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.repository
            .delete_api_key(&*tx, ApiKeyDeleteParams { id })
            .await?;

        tx.commit().await?;

        Ok(())
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
    pub created_at: Option<DateTime<Utc>>,
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
