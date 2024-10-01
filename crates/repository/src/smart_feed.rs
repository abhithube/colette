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
    prelude::{Json, Uuid},
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, DbErr, IntoActiveModel, SqlErr,
    TransactionError, TransactionTrait,
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
        self.db
            .transaction::<_, SmartFeed, Error>(|txn| {
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

                    find_by_id(txn, IdParams::new(model.last_insert_id, data.profile_id)).await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }
}

#[async_trait::async_trait]
impl Updatable for SmartFeedSqlRepository {
    type Params = IdParams;
    type Data = SmartFeedUpdateData;
    type Output = Result<SmartFeed, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        self.db
            .transaction::<_, SmartFeed, Error>(|txn| {
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

                    find_by_id(txn, params).await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
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

pub(crate) async fn find<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
) -> Result<Vec<SmartFeed>, Error> {
    let feeds = query::smart_feed::select(db, id, profile_id, limit, cursor)
        .await
        .map(|e| e.into_iter().map(SmartFeed::from).collect())
        .map_err(|e| Error::Unknown(e.into()))?;

    Ok(feeds)
}

async fn find_by_id<Db: ConnectionTrait>(db: &Db, params: IdParams) -> Result<SmartFeed, Error> {
    let mut feeds = find(db, Some(params.id), params.profile_id, None, None).await?;
    if feeds.is_empty() {
        return Err(Error::NotFound(params.id));
    }

    Ok(feeds.swap_remove(0))
}

struct Op {
    r#type: Operation,
    negated: Option<bool>,
    value: Json,
}

impl From<TextOperation> for Op {
    fn from(value: TextOperation) -> Self {
        match value {
            TextOperation::Equals(value) => Self {
                r#type: Operation::Equals,
                negated: None,
                value: Json::String(value),
            },
            TextOperation::DoesNotEqual(value) => Self {
                r#type: Operation::Equals,
                negated: Some(true),
                value: Json::String(value),
            },
            TextOperation::Contains(value) => Self {
                r#type: Operation::Contains,
                negated: None,
                value: Json::String(value),
            },
            TextOperation::DoesNotContain(value) => Self {
                r#type: Operation::Contains,
                negated: Some(true),
                value: Json::String(value),
            },
        }
    }
}

impl From<DateOperation> for Op {
    fn from(value: DateOperation) -> Self {
        match value {
            DateOperation::Equals(value) => Self {
                r#type: Operation::Equals,
                negated: None,
                value: Json::String(value.to_rfc3339()),
            },
            DateOperation::GreaterThan(value) => Self {
                r#type: Operation::GreaterThan,
                negated: None,
                value: Json::String(value.to_rfc3339()),
            },
            DateOperation::LessThan(value) => Self {
                r#type: Operation::LessThan,
                negated: None,
                value: Json::String(value.to_rfc3339()),
            },
            DateOperation::InLast(value) => Self {
                r#type: Operation::InLastMillis,
                negated: None,
                value: Json::Number(value.into()),
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
                        r#type: Operation::Equals,
                        negated: None,
                        value: Json::Bool(op.value),
                    },
                ),
            };

            query::smart_feed_filter::InsertMany {
                id: Uuid::new_v4(),
                field,
                operation: op.r#type,
                is_negated: op.negated,
                value: op.value,
            }
        })
        .collect::<Vec<_>>();

    query::smart_feed_filter::insert_many(db, insert_data, smart_feed_id, profile_id).await?;

    Ok(())
}
