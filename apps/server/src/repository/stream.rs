use colette_core::{
    FeedEntry, Stream,
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    stream::{
        Error, StreamCreateData, StreamEntryFindParams, StreamFindParams, StreamRepository,
        StreamUpdateData,
    },
};
use colette_model::streams;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    IntoActiveModel, ModelTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait, RuntimeErr,
    TransactionTrait,
};
use sqlx::QueryBuilder;
use uuid::Uuid;

use super::{common::ToSql, feed_entry::FeedEntryRow};

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
impl Findable for SqliteStreamRepository {
    type Params = StreamFindParams;
    type Output = Result<Vec<Stream>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let streams = streams::Entity::find()
            .filter(streams::Column::UserId.eq(params.user_id.to_string()))
            .apply_if(params.id, |query, id| {
                query.filter(streams::Column::Id.eq(id.to_string()))
            })
            .apply_if(params.cursor, |query, cursor| {
                query.filter(streams::Column::Title.gt(cursor.title))
            })
            .order_by_asc(streams::Column::Title)
            .limit(params.limit.map(|e| e as u64))
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(streams)
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteStreamRepository {
    type Data = StreamCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let id = Uuid::new_v4();
        let stream = streams::ActiveModel {
            id: ActiveValue::Set(id.into()),
            title: ActiveValue::Set(data.title.clone()),
            filter_raw: ActiveValue::Set(serde_json::to_string(&data.filter).unwrap()),
            user_id: ActiveValue::Set(data.user_id.into()),
            ..Default::default()
        };
        stream.insert(&self.db).await.map_err(|e| match e {
            DbErr::RecordNotInserted => Error::Conflict(data.title),
            _ => Error::Database(e),
        })?;

        Ok(id)
    }
}

#[async_trait::async_trait]
impl Updatable for SqliteStreamRepository {
    type Params = IdParams;
    type Data = StreamUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let tx = self.db.begin().await?;

        let Some(stream) = streams::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if stream.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        let mut stream = stream.into_active_model();

        if let Some(title) = data.title {
            stream.title = ActiveValue::Set(title);
        }
        if let Some(filter) = data.filter {
            stream.filter_raw = ActiveValue::Set(serde_json::to_string(&filter).unwrap());
        }

        if stream.is_changed() {
            stream.update(&tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for SqliteStreamRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let tx = self.db.begin().await?;

        let Some(stream) = streams::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if stream.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        stream.delete(&tx).await?;

        tx.commit().await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl StreamRepository for SqliteStreamRepository {
    async fn find_entries(&self, params: StreamEntryFindParams) -> Result<Vec<FeedEntry>, Error> {
        let initial = format!(
            r#"SELECT ufe.id,
       fe.link,
       fe.title,
       fe.published_at,
       fe.description,
       fe.author,
       fe.thumbnail_url,
       ufe.has_read,
       ufe.user_feed_id AS feed_id,
       ufe.created_at,
       ufe.updated_at
  FROM user_feed_entries ufe
  JOIN feed_entries fe on fe.id = ufe.feed_entry_id
 WHERE ufe.user_id = '{}'"#,
            params.user_id
        );

        let mut qb = QueryBuilder::new(initial);

        let where_clause = params.filter.to_sql();
        if !where_clause.is_empty() {
            qb.push(" AND ");
            qb.push(&where_clause);
        }

        if let Some(cursor) = params.cursor {
            qb.push(" AND (fe.published_at, ufe.id) > (");

            let mut separated = qb.separated(", ");
            separated.push_bind(cursor.published_at);
            separated.push_bind(cursor.id);
            separated.push_unseparated(")");
        }

        qb.push("\n ORDER BY fe.published_at DESC, ufe.id DESC");

        if let Some(limit) = params.limit {
            qb.push("\n LIMIT ");
            qb.push_bind(limit);
        }

        let query = qb.build_query_as::<FeedEntryRow>();

        let entries = query
            .fetch_all(self.db.get_sqlite_connection_pool())
            .await
            .map(|e| e.into_iter().map(Into::into).collect())
            .map_err(|e| DbErr::Query(RuntimeErr::SqlxError(e)))?;

        Ok(entries)
    }
}
