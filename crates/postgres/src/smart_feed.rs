use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    smart_feed::{
        Cursor, DateOperation, Error, SmartFeedCreateData, SmartFeedFilter, SmartFeedRepository,
        SmartFeedUpdateData, TextOperation,
    },
    SmartFeed,
};
use colette_sql::smart_feed_filter::{Field, Operation};
use sqlx::{types::Uuid, PgExecutor, PgPool};

use crate::query;

pub struct PostgresSmartFeedRepository {
    pub(crate) pool: PgPool,
}

impl PostgresSmartFeedRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresSmartFeedRepository {
    type Params = IdParams;
    type Output = Result<SmartFeed, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        find_by_id(&self.pool, params).await
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresSmartFeedRepository {
    type Data = SmartFeedCreateData;
    type Output = Result<SmartFeed, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let id = query::smart_feed::insert(
            &mut *tx,
            Uuid::new_v4(),
            data.title.clone(),
            data.profile_id,
        )
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(data.title),
            _ => Error::Unknown(e.into()),
        })?;

        if let Some(filters) = data.filters {
            insert_filters(&mut *tx, filters, id, data.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        let feed = find_by_id(&mut *tx, IdParams::new(id, data.profile_id)).await?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(feed)
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresSmartFeedRepository {
    type Params = IdParams;
    type Data = SmartFeedUpdateData;
    type Output = Result<SmartFeed, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.title.is_some() {
            query::smart_feed::update(&mut *tx, params.id, params.profile_id, data.title)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound(params.id),
                    _ => Error::Unknown(e.into()),
                })?;
        }

        if let Some(filters) = data.filters {
            query::smart_feed_filter::delete_many(&mut *tx, params.id, params.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            insert_filters(&mut *tx, filters, params.id, params.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        let feed = find_by_id(&mut *tx, params)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(feed)
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresSmartFeedRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        query::smart_feed::delete(&self.pool, params.id, params.profile_id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })
    }
}

#[async_trait::async_trait]
impl SmartFeedRepository for PostgresSmartFeedRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
    ) -> Result<Vec<SmartFeed>, Error> {
        find(&self.pool, None, profile_id, limit, cursor).await
    }
}

pub(crate) async fn find(
    executor: impl PgExecutor<'_>,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
) -> Result<Vec<SmartFeed>, Error> {
    query::smart_feed::select(executor, id, profile_id, cursor, limit)
        .await
        .map_err(|e| Error::Unknown(e.into()))
}

async fn find_by_id(executor: impl PgExecutor<'_>, params: IdParams) -> Result<SmartFeed, Error> {
    let mut feeds = find(executor, Some(params.id), params.profile_id, None, None).await?;
    if feeds.is_empty() {
        return Err(Error::NotFound(params.id));
    }

    Ok(feeds.swap_remove(0))
}

struct Op {
    r#type: Operation,
    value: String,
}

impl From<TextOperation> for Op {
    fn from(value: TextOperation) -> Self {
        match value {
            TextOperation::Equals(value) => Self {
                r#type: Operation::Eq,
                value,
            },
            TextOperation::DoesNotEqual(value) => Self {
                r#type: Operation::Ne,
                value,
            },
            TextOperation::Contains(value) => Self {
                r#type: Operation::Like,
                value,
            },
            TextOperation::DoesNotContain(value) => Self {
                r#type: Operation::NotLike,
                value,
            },
        }
    }
}

impl From<DateOperation> for Op {
    fn from(value: DateOperation) -> Self {
        match value {
            DateOperation::Equals(value) => Self {
                r#type: Operation::Eq,
                value: value.to_rfc3339(),
            },
            DateOperation::GreaterThan(value) => Self {
                r#type: Operation::GreaterThan,
                value: value.to_rfc3339(),
            },
            DateOperation::LessThan(value) => Self {
                r#type: Operation::LessThan,
                value: value.to_rfc3339(),
            },
            DateOperation::InLast(value) => Self {
                r#type: Operation::InLastXSec,
                value: value.to_string(),
            },
        }
    }
}

async fn insert_filters(
    executor: impl PgExecutor<'_>,
    filters: Vec<SmartFeedFilter>,
    smart_feed_id: Uuid,
    profile_id: Uuid,
) -> Result<(), sqlx::Error> {
    let insert_data = filters
        .into_iter()
        .map(|e| {
            let (field, op): (Field, Op) = match e {
                SmartFeedFilter::Link(op) => (Field::Link, op.into()),
                SmartFeedFilter::Title(op) => (Field::Title, op.into()),
                SmartFeedFilter::PublishedAt(op) => (Field::PublishedAt, op.into()),
                SmartFeedFilter::Description(op) => (Field::Description, op.into()),
                SmartFeedFilter::Author(op) => (Field::Author, op.into()),
                SmartFeedFilter::HasRead(op) => (
                    Field::HasRead,
                    Op {
                        r#type: Operation::Eq,
                        value: op.value.to_string(),
                    },
                ),
            };

            colette_sql::smart_feed_filter::InsertMany {
                id: Uuid::new_v4(),
                field,
                operation: op.r#type,
                value: op.value,
            }
        })
        .collect::<Vec<_>>();

    query::smart_feed_filter::insert_many(executor, insert_data, smart_feed_id, profile_id).await?;

    Ok(())
}
