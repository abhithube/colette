use colette_core::{
    Collection,
    collection::{
        CollectionById, CollectionCreateParams, CollectionDeleteParams, CollectionFindByIdParams,
        CollectionFindParams, CollectionRepository, CollectionUpdateParams, Error,
    },
    common::Transaction,
};
use colette_model::{CollectionRow, collections};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr, FromQueryResult,
    sea_query::{Asterisk, Expr, Order, Query},
};

#[derive(Debug, Clone)]
pub struct SqliteCollectionRepository {
    db: DatabaseConnection,
}

impl SqliteCollectionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl CollectionRepository for SqliteCollectionRepository {
    async fn find_collections(
        &self,
        params: CollectionFindParams,
    ) -> Result<Vec<Collection>, Error> {
        let mut query = Query::select()
            .column(Asterisk)
            .from(collections::Entity)
            .apply_if(params.id, |query, id| {
                query.and_where(
                    Expr::col((collections::Entity, collections::Column::Id)).eq(id.to_string()),
                );
            })
            .apply_if(params.user_id, |query, user_id| {
                query.and_where(
                    Expr::col((collections::Entity, collections::Column::UserId))
                        .eq(user_id.to_string()),
                );
            })
            .apply_if(params.cursor, |query, cursor| {
                query.and_where(
                    Expr::col((collections::Entity, collections::Column::Title))
                        .gt(Expr::val(cursor.title)),
                );
            })
            .order_by(
                (collections::Entity, collections::Column::Title),
                Order::Asc,
            )
            .to_owned();

        if let Some(limit) = params.limit {
            query.limit(limit as u64);
        }

        let collections =
            CollectionRow::find_by_statement(self.db.get_database_backend().build(&query))
                .all(&self.db)
                .await
                .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(collections)
    }

    async fn find_collection_by_id(
        &self,
        tx: &dyn Transaction,
        params: CollectionFindByIdParams,
    ) -> Result<CollectionById, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let query = Query::select()
            .column((collections::Entity, collections::Column::Id))
            .column((collections::Entity, collections::Column::UserId))
            .from(collections::Entity)
            .and_where(
                Expr::col((collections::Entity, collections::Column::Id)).eq(params.id.to_string()),
            )
            .to_owned();

        let Some(result) = tx
            .query_one(self.db.get_database_backend().build(&query))
            .await?
        else {
            return Err(Error::NotFound(params.id));
        };

        Ok(CollectionById {
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

    async fn create_collection(&self, params: CollectionCreateParams) -> Result<(), Error> {
        let query = Query::insert()
            .columns([
                collections::Column::Id,
                collections::Column::Title,
                collections::Column::FilterRaw,
                collections::Column::UserId,
            ])
            .values_panic([
                params.id.to_string().into(),
                params.title.clone().into(),
                serde_json::to_string(&params.filter).unwrap().into(),
                params.user_id.to_string().into(),
            ])
            .to_owned();

        self.db
            .execute(self.db.get_database_backend().build(&query))
            .await
            .map_err(|e| match e {
                DbErr::RecordNotInserted => Error::Conflict(params.title),
                _ => Error::Database(e),
            })?;

        Ok(())
    }

    async fn update_collection(
        &self,
        tx: &dyn Transaction,
        params: CollectionUpdateParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        if params.title.is_none() && params.filter.is_none() {
            return Ok(());
        }

        let mut query = Query::update()
            .table(collections::Entity)
            .and_where(Expr::col(collections::Column::Id).eq(params.id.to_string()))
            .to_owned();

        if let Some(title) = params.title {
            query.value(collections::Column::Title, title);
        }
        if let Some(filter) = params.filter {
            query.value(
                collections::Column::FilterRaw,
                serde_json::to_string(&filter).unwrap(),
            );
        }

        tx.execute(self.db.get_database_backend().build(&query))
            .await?;

        Ok(())
    }

    async fn delete_collection(
        &self,
        tx: &dyn Transaction,
        params: CollectionDeleteParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let query = Query::delete()
            .from_table(collections::Entity)
            .and_where(Expr::col(collections::Column::Id).eq(params.id.to_string()))
            .to_owned();

        tx.execute(self.db.get_database_backend().build(&query))
            .await?;

        Ok(())
    }
}
