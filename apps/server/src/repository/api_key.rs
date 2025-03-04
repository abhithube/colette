use colette_core::{
    ApiKey,
    api_key::{
        ApiKeyById, ApiKeyCreateData, ApiKeyFindParams, ApiKeyRepository, ApiKeySearchParams,
        ApiKeySearched, ApiKeyUpdateData, Error,
    },
    common::Transaction,
};
use colette_model::api_keys;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DatabaseTransaction,
    EntityTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
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
            .apply_if(params.id, |query, id| {
                query.filter(api_keys::Column::Id.eq(id.to_string()))
            })
            .apply_if(params.user_id, |query, user_id| {
                query.filter(api_keys::Column::UserId.eq(user_id.to_string()))
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

    async fn find_api_key_by_id(
        &self,
        tx: &dyn Transaction,
        id: Uuid,
    ) -> Result<ApiKeyById, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let Some((id, user_id)) = api_keys::Entity::find()
            .select_only()
            .columns([api_keys::Column::Id, api_keys::Column::UserId])
            .filter(api_keys::Column::Id.eq(id.to_string()))
            .into_tuple::<(String, String)>()
            .one(tx)
            .await?
        else {
            return Err(Error::NotFound(id));
        };

        Ok(ApiKeyById {
            id: id.parse().unwrap(),
            user_id: user_id.parse().unwrap(),
        })
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

    async fn update_api_key(
        &self,
        tx: &dyn Transaction,
        id: Uuid,
        data: ApiKeyUpdateData,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let mut model = api_keys::ActiveModel {
            id: ActiveValue::Unchanged(id.into()),
            ..Default::default()
        };

        if let Some(title) = data.title {
            model.title = ActiveValue::Set(title);
        }

        if model.is_changed() {
            model.update(tx).await?;
        }

        Ok(())
    }

    async fn delete_api_key(&self, tx: &dyn Transaction, id: Uuid) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        api_keys::Entity::delete_by_id(id).exec(tx).await?;

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
