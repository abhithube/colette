use colette_core::{
    ApiKey,
    api_key::{
        ApiKeyById, ApiKeyCreateParams, ApiKeyDeleteParams, ApiKeyFindByIdParams, ApiKeyFindParams,
        ApiKeyRepository, ApiKeySearchParams, ApiKeySearched, ApiKeyUpdateParams, Error,
    },
    common::Transaction,
};
use colette_query::{IntoDelete, IntoInsert, IntoSelect, IntoUpdate};
use sea_orm::{ConnectionTrait, DatabaseConnection, DatabaseTransaction, FromQueryResult};

use super::common::parse_timestamp;

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
        let api_keys = ApiKeyRow::find_by_statement(
            self.db.get_database_backend().build(&params.into_select()),
        )
        .all(&self.db)
        .await
        .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(api_keys)
    }

    async fn find_api_key_by_id(
        &self,
        tx: &dyn Transaction,
        params: ApiKeyFindByIdParams,
    ) -> Result<ApiKeyById, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let id = params.id;

        let Some(result) = tx
            .query_one(self.db.get_database_backend().build(&params.into_select()))
            .await?
        else {
            return Err(Error::NotFound(id));
        };

        Ok(ApiKeyById {
            id: result
                .try_get_by_index::<String>(0)
                .unwrap()
                .parse()
                .unwrap(),
            user_id: result
                .try_get_by_index::<String>(1)
                .unwrap()
                .parse()
                .unwrap(),
        })
    }

    async fn create_api_key(&self, params: ApiKeyCreateParams) -> Result<(), Error> {
        self.db
            .execute(self.db.get_database_backend().build(&params.into_insert()))
            .await?;

        Ok(())
    }

    async fn update_api_key(
        &self,
        tx: &dyn Transaction,
        params: ApiKeyUpdateParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        if params.title.is_none() {
            return Ok(());
        }

        tx.execute(self.db.get_database_backend().build(&params.into_update()))
            .await?;

        Ok(())
    }

    async fn delete_api_key(
        &self,
        tx: &dyn Transaction,
        params: ApiKeyDeleteParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        tx.execute(self.db.get_database_backend().build(&params.into_delete()))
            .await?;

        Ok(())
    }

    async fn search_api_key(
        &self,
        params: ApiKeySearchParams,
    ) -> Result<Option<ApiKeySearched>, Error> {
        let result = self
            .db
            .query_one(self.db.get_database_backend().build(&params.into_select()))
            .await?;

        Ok(result.map(|e| ApiKeySearched {
            verification_hash: e.try_get_by_index::<String>(0).unwrap(),
            user_id: e.try_get_by_index::<String>(1).unwrap().parse().unwrap(),
        }))
    }
}

#[derive(sea_orm::FromQueryResult)]
struct ApiKeyRow {
    id: String,
    title: String,
    preview: String,
    user_id: String,
    created_at: i32,
    updated_at: i32,
}

impl From<ApiKeyRow> for ApiKey {
    fn from(value: ApiKeyRow) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            title: value.title,
            preview: value.preview,
            user_id: value.user_id.parse().unwrap(),
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
        }
    }
}
