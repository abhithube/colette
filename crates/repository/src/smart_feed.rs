use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    smart_feed::{
        Cursor, DateOperation, Error, SmartFeedCreateData, SmartFeedFilter, SmartFeedRepository,
        SmartFeedUpdateData, TextOperation,
    },
    SmartFeed,
};
use colette_entity::sea_orm_active_enums::{Field, Operation};
use sea_orm::{
    prelude::Uuid, ActiveModelTrait, ConnectionTrait, DatabaseConnection, DbErr, IntoActiveModel,
    SqlErr, TransactionError, TransactionTrait,
};

use crate::query;

pub struct SmartFeedSqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl SmartFeedSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for SmartFeedSqlRepository {
    type Params = IdParams;
    type Output = Result<SmartFeed, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        find_by_id(&self.db, params).await
    }
}

#[async_trait::async_trait]
impl Creatable for SmartFeedSqlRepository {
    type Data = SmartFeedCreateData;
    type Output = Result<SmartFeed, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let id = self
            .db
            .transaction::<_, Uuid, Error>(|txn| {
                Box::pin(async move {
                    let model = query::smart_feed::insert(
                        txn,
                        Uuid::new_v4(),
                        data.title.clone(),
                        data.profile_id,
                    )
                    .await
                    .map_err(|e| match e.sql_err() {
                        Some(SqlErr::UniqueConstraintViolation(_)) => Error::Conflict(data.title),
                        _ => Error::Unknown(e.into()),
                    })?;

                    if let Some(filters) = data.filters {
                        insert_filters(txn, filters, model.last_insert_id, data.profile_id)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    Ok(model.last_insert_id)
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })?;

        find_by_id(&self.db, IdParams::new(id, data.profile_id)).await
    }
}

#[async_trait::async_trait]
impl Updatable for SmartFeedSqlRepository {
    type Params = IdParams;
    type Data = SmartFeedUpdateData;
    type Output = Result<SmartFeed, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        self.db
            .transaction::<_, (), Error>(|txn| {
                Box::pin(async move {
                    let Some(model) =
                        query::smart_feed::select_by_id(txn, params.id, params.profile_id)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };

                    let smart_feed_id = model.id;

                    let mut active_model = model.into_active_model();
                    if let Some(title) = data.title {
                        active_model.title.set_if_not_equals(title);
                    }

                    if active_model.is_changed() {
                        active_model
                            .update(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    if let Some(filters) = data.filters {
                        query::smart_feed_filter::delete_many_by_smart_feed(
                            txn,
                            smart_feed_id,
                            params.profile_id,
                        )
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                        insert_filters(txn, filters, smart_feed_id, params.profile_id)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })?;

        find_by_id(&self.db, params).await
    }
}

#[async_trait::async_trait]
impl Deletable for SmartFeedSqlRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let result = query::smart_feed::delete_by_id(&self.db, params.id, params.profile_id)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl SmartFeedRepository for SmartFeedSqlRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
    ) -> Result<Vec<SmartFeed>, Error> {
        find(&self.db, None, profile_id, limit, cursor).await
    }
}

pub(crate) async fn find(
    db: &DatabaseConnection,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
) -> Result<Vec<SmartFeed>, Error> {
    colette_postgres::smart_feed::select(
        db.get_postgres_connection_pool(),
        id,
        profile_id,
        cursor,
        limit,
    )
    .await
    .map_err(|e| Error::Unknown(e.into()))
}

async fn find_by_id(db: &DatabaseConnection, params: IdParams) -> Result<SmartFeed, Error> {
    let mut feeds = find(db, Some(params.id), params.profile_id, None, None).await?;
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
                r#type: Operation::U003D,
                value,
            },
            TextOperation::DoesNotEqual(value) => Self {
                r#type: Operation::U0021U003D,
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
                r#type: Operation::U003D,
                value: value.to_rfc3339(),
            },
            DateOperation::GreaterThan(value) => Self {
                r#type: Operation::U003E,
                value: value.to_rfc3339(),
            },
            DateOperation::LessThan(value) => Self {
                r#type: Operation::U003C,
                value: value.to_rfc3339(),
            },
            DateOperation::InLast(value) => Self {
                r#type: Operation::InLastXSec,
                value: value.to_string(),
            },
        }
    }
}

async fn insert_filters<DB: ConnectionTrait>(
    db: &DB,
    filters: Vec<SmartFeedFilter>,
    smart_feed_id: Uuid,
    profile_id: Uuid,
) -> Result<(), DbErr> {
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
                        r#type: Operation::U003D,
                        value: op.value.to_string(),
                    },
                ),
            };

            query::smart_feed_filter::InsertMany {
                id: Uuid::new_v4(),
                field,
                operation: op.r#type,
                value: op.value,
            }
        })
        .collect::<Vec<_>>();

    query::smart_feed_filter::insert_many(db, insert_data, smart_feed_id, profile_id).await?;

    Ok(())
}
