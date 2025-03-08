use colette_core::{
    Stream,
    common::Transaction,
    stream::{
        Error, StreamById, StreamCreateParams, StreamDeleteParams, StreamFindByIdParams,
        StreamFindParams, StreamRepository, StreamUpdateParams,
    },
};
use colette_model::{StreamRow, streams};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr, FromQueryResult,
    sea_query::{Asterisk, Expr, Order, Query},
};

#[derive(Debug, Clone)]
pub struct SqliteStreamRepository {
    db: DatabaseConnection,
}

impl SqliteStreamRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl StreamRepository for SqliteStreamRepository {
    async fn find_streams(&self, params: StreamFindParams) -> Result<Vec<Stream>, Error> {
        let mut query = Query::select()
            .column(Asterisk)
            .from(streams::Entity)
            .apply_if(params.id, |query, id| {
                query.and_where(
                    Expr::col((streams::Entity, streams::Column::Id)).eq(id.to_string()),
                );
            })
            .apply_if(params.user_id, |query, user_id| {
                query.and_where(
                    Expr::col((streams::Entity, streams::Column::UserId)).eq(user_id.to_string()),
                );
            })
            .apply_if(params.cursor, |query, cursor| {
                query.and_where(
                    Expr::col((streams::Entity, streams::Column::Title))
                        .gt(Expr::val(cursor.title)),
                );
            })
            .order_by((streams::Entity, streams::Column::Title), Order::Asc)
            .to_owned();

        if let Some(limit) = params.limit {
            query.limit(limit as u64);
        }

        let streams = StreamRow::find_by_statement(self.db.get_database_backend().build(&query))
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(streams)
    }

    async fn find_stream_by_id(
        &self,
        tx: &dyn Transaction,
        params: StreamFindByIdParams,
    ) -> Result<StreamById, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let query = Query::select()
            .column((streams::Entity, streams::Column::Id))
            .column((streams::Entity, streams::Column::UserId))
            .from(streams::Entity)
            .and_where(Expr::col((streams::Entity, streams::Column::Id)).eq(params.id.to_string()))
            .to_owned();

        let Some(result) = tx
            .query_one(self.db.get_database_backend().build(&query))
            .await?
        else {
            return Err(Error::NotFound(params.id));
        };

        Ok(StreamById {
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

    async fn create_stream(&self, params: StreamCreateParams) -> Result<(), Error> {
        let query = Query::insert()
            .columns([
                streams::Column::Id,
                streams::Column::Title,
                streams::Column::FilterRaw,
                streams::Column::UserId,
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

    async fn update_stream(
        &self,
        tx: &dyn Transaction,
        params: StreamUpdateParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        if params.title.is_none() && params.filter.is_none() {
            return Ok(());
        }

        let mut query = Query::update()
            .table(streams::Entity)
            .and_where(Expr::col(streams::Column::Id).eq(params.id.to_string()))
            .to_owned();

        if let Some(title) = params.title {
            query.value(streams::Column::Title, title);
        }
        if let Some(filter) = params.filter {
            query.value(
                streams::Column::FilterRaw,
                serde_json::to_string(&filter).unwrap(),
            );
        }

        tx.execute(self.db.get_database_backend().build(&query))
            .await?;

        Ok(())
    }

    async fn delete_stream(
        &self,
        tx: &dyn Transaction,
        params: StreamDeleteParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let query = Query::delete()
            .from_table(streams::Entity)
            .and_where(Expr::col(streams::Column::Id).eq(params.id.to_string()))
            .to_owned();

        tx.execute(self.db.get_database_backend().build(&query))
            .await?;

        Ok(())
    }
}
