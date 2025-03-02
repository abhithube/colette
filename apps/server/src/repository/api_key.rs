use colette_core::{
    ApiKey,
    api_key::{
        ApiKeyCreateData, ApiKeyFindParams, ApiKeyRepository, ApiKeySearchParams, ApiKeySearched,
        ApiKeyUpdateData, Error,
    },
    common::IdParams,
};
use colette_model::api_keys;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    ModelTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait, TransactionTrait,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteApiKeyRepository {
    db: DatabaseConnection,
}

impl SqliteApiKeyRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl ApiKeyRepository for SqliteApiKeyRepository {
    async fn find_api_keys(&self, params: ApiKeyFindParams) -> Result<Vec<ApiKey>, Error> {
        let api_keys = api_keys::Entity::find()
            .filter(api_keys::Column::UserId.eq(params.user_id.to_string()))
            .apply_if(params.id, |query, id| {
                query.filter(api_keys::Column::Id.eq(id.to_string()))
            })
            .apply_if(params.cursor, |query, cursor| {
                query.filter(api_keys::Column::CreatedAt.gt(cursor.created_at.to_rfc3339()))
            })
            .order_by_asc(api_keys::Column::CreatedAt)
            .limit(params.limit.map(|e| e as u64))
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(api_keys)
    }

    async fn create_api_key(&self, data: ApiKeyCreateData) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();

        let api_key = api_keys::ActiveModel {
            id: ActiveValue::Set(id.into()),
            lookup_hash: ActiveValue::Set(data.lookup_hash),
            verification_hash: ActiveValue::Set(data.verification_hash),
            title: ActiveValue::Set(data.title),
            preview: ActiveValue::Set(data.preview),
            user_id: ActiveValue::Set(data.user_id.into()),
            ..Default::default()
        };
        api_key.insert(&self.db).await?;

        Ok(id)
    }

    async fn update_api_key(&self, params: IdParams, data: ApiKeyUpdateData) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let Some(api_key) = api_keys::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if api_key.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        let mut api_key = api_key.into_active_model();

        if let Some(title) = data.title {
            api_key.title = ActiveValue::Set(title);
        }

        if api_key.is_changed() {
            api_key.update(&tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn delete_api_key(&self, params: IdParams) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let Some(api_key) = api_keys::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if api_key.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        api_key.delete(&tx).await?;

        tx.commit().await?;

        Ok(())
    }

    async fn search_api_key(
        &self,
        params: ApiKeySearchParams,
    ) -> Result<Option<ApiKeySearched>, Error> {
        let api_key = api_keys::Entity::find()
            .filter(api_keys::Column::LookupHash.eq(params.lookup_hash))
            .one(&self.db)
            .await
            .map(|e| e.map(Into::into))?;

        Ok(api_key)
    }
}
