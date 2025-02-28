use colette_core::{
    ApiKey,
    api_key::{
        ApiKeyCreateData, ApiKeyFindParams, ApiKeyRepository, ApiKeySearchParams, ApiKeySearched,
        ApiKeyUpdateData, Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
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
impl Findable for SqliteApiKeyRepository {
    type Params = ApiKeyFindParams;
    type Output = Result<Vec<ApiKey>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
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
}

#[async_trait::async_trait]
impl Creatable for SqliteApiKeyRepository {
    type Data = ApiKeyCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
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
}

#[async_trait::async_trait]
impl Updatable for SqliteApiKeyRepository {
    type Params = IdParams;
    type Data = ApiKeyUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
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
}

#[async_trait::async_trait]
impl Deletable for SqliteApiKeyRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
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
}

#[async_trait::async_trait]
impl ApiKeyRepository for SqliteApiKeyRepository {
    async fn search(&self, params: ApiKeySearchParams) -> Result<Option<ApiKeySearched>, Error> {
        let api_key = api_keys::Entity::find()
            .filter(api_keys::Column::LookupHash.eq(params.lookup_hash))
            .one(&self.db)
            .await
            .map(|e| e.map(Into::into))?;

        Ok(api_key)
    }
}
