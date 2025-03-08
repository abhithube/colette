use colette_core::{
    ApiKey,
    api_key::{
        ApiKeyById, ApiKeyCreateData, ApiKeyFindParams, ApiKeyRepository, ApiKeySearchParams,
        ApiKeySearched, ApiKeyUpdateData, Error,
    },
    common::Transaction,
};
use colette_model::{ApiKeyRow, api_keys};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, FromQueryResult,
    sea_query::{Asterisk, Expr, Order, Query},
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
        let mut query = Query::select()
            .column(Asterisk)
            .from(api_keys::Entity)
            .apply_if(params.id, |query, id| {
                query.and_where(
                    Expr::col((api_keys::Entity, api_keys::Column::Id)).eq(id.to_string()),
                );
            })
            .apply_if(params.user_id, |query, user_id| {
                query.and_where(
                    Expr::col((api_keys::Entity, api_keys::Column::UserId)).eq(user_id.to_string()),
                );
            })
            .apply_if(params.cursor, |query, cursor| {
                query.and_where(
                    Expr::col((api_keys::Entity, api_keys::Column::CreatedAt))
                        .gt(Expr::val(cursor.created_at.timestamp())),
                );
            })
            .order_by((api_keys::Entity, api_keys::Column::CreatedAt), Order::Asc)
            .to_owned();

        if let Some(limit) = params.limit {
            query.limit(limit as u64);
        }

        let api_keys = ApiKeyRow::find_by_statement(self.db.get_database_backend().build(&query))
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

        let query = Query::select()
            .column((api_keys::Entity, api_keys::Column::Id))
            .column((api_keys::Entity, api_keys::Column::UserId))
            .from(api_keys::Entity)
            .and_where(Expr::col((api_keys::Entity, api_keys::Column::Id)).eq(id.to_string()))
            .to_owned();

        let Some(result) = tx
            .query_one(self.db.get_database_backend().build(&query))
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

    async fn create_api_key(&self, data: ApiKeyCreateData) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();

        let query = Query::insert()
            .columns([
                api_keys::Column::Id,
                api_keys::Column::LookupHash,
                api_keys::Column::VerificationHash,
                api_keys::Column::Title,
                api_keys::Column::Preview,
                api_keys::Column::UserId,
            ])
            .values_panic([
                id.to_string().into(),
                data.lookup_hash.into(),
                data.verification_hash.into(),
                data.title.into(),
                data.preview.into(),
                data.user_id.to_string().into(),
            ])
            .to_owned();

        self.db
            .execute(self.db.get_database_backend().build(&query))
            .await?;

        Ok(id)
    }

    async fn update_api_key(
        &self,
        tx: &dyn Transaction,
        id: Uuid,
        data: ApiKeyUpdateData,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        if data.title.is_none() {
            return Ok(());
        }

        let mut query = Query::update()
            .table(api_keys::Entity)
            .and_where(Expr::col(api_keys::Column::Id).eq(id.to_string()))
            .to_owned();

        if let Some(title) = data.title {
            query.value(api_keys::Column::Title, title);
        }

        tx.execute(self.db.get_database_backend().build(&query))
            .await?;

        Ok(())
    }

    async fn delete_api_key(&self, tx: &dyn Transaction, id: Uuid) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let query = Query::delete()
            .from_table(api_keys::Entity)
            .and_where(Expr::col(api_keys::Column::Id).eq(id.to_string()))
            .to_owned();

        tx.execute(self.db.get_database_backend().build(&query))
            .await?;

        Ok(())
    }

    async fn search_api_key(
        &self,
        params: ApiKeySearchParams,
    ) -> Result<Option<ApiKeySearched>, Error> {
        let query = Query::select()
            .column((api_keys::Entity, api_keys::Column::VerificationHash))
            .column((api_keys::Entity, api_keys::Column::UserId))
            .from(api_keys::Entity)
            .and_where(
                Expr::col((api_keys::Entity, api_keys::Column::LookupHash)).eq(params.lookup_hash),
            )
            .to_owned();

        let result = self
            .db
            .query_one(self.db.get_database_backend().build(&query))
            .await?;

        Ok(result.map(|e| ApiKeySearched {
            verification_hash: e.try_get_by_index::<String>(0).unwrap(),
            user_id: e.try_get_by_index::<String>(1).unwrap().parse().unwrap(),
        }))
    }
}
