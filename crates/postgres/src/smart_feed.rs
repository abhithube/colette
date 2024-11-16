use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    smart_feed::{
        Cursor, DateOperation, Error, SmartFeedCreateData, SmartFeedFilter as Filter,
        SmartFeedRepository, SmartFeedUpdateData, TextOperation,
    },
    SmartFeed,
};
use colette_sql::{
    feed_entry::FeedEntry,
    profile_feed_entry::ProfileFeedEntry,
    smart_feed_filter::{Field, Operation, SmartFeedFilter},
};
use sea_query::{Alias, CaseStatement, Expr, Func, Iden, PostgresQueryBuilder, SimpleExpr};
use sea_query_binder::SqlxBinder;
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresSmartFeedRepository {
    pool: PgPool,
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
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let id = {
            let (sql, values) =
                colette_sql::smart_feed::insert(None, data.title.clone(), data.profile_id)
                    .build_sqlx(PostgresQueryBuilder);

            sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| match e {
                    sqlx::Error::Database(e) if e.is_unique_violation() => {
                        Error::Conflict(data.title)
                    }
                    _ => Error::Unknown(e.into()),
                })?
        };

        if let Some(filters) = data.filters {
            insert_filters(&mut *tx, filters, id, data.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(id)
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresSmartFeedRepository {
    type Params = IdParams;
    type Data = SmartFeedUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.title.is_some() {
            let count = {
                let (sql, values) =
                    colette_sql::smart_feed::update(params.id, params.profile_id, data.title)
                        .build_sqlx(PostgresQueryBuilder);

                sqlx::query_with(&sql, values)
                    .execute(&mut *tx)
                    .await
                    .map(|e| e.rows_affected())
                    .map_err(|e| Error::Unknown(e.into()))?
            };
            if count == 0 {
                return Err(Error::NotFound(params.id));
            }
        }

        if let Some(filters) = data.filters {
            {
                let (sql, values) =
                    colette_sql::smart_feed_filter::delete_many(params.profile_id, params.id)
                        .build_sqlx(PostgresQueryBuilder);

                sqlx::query_with(&sql, values)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;
            }

            insert_filters(&mut *tx, filters, params.id, params.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresSmartFeedRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let count = {
            let (sql, values) = colette_sql::smart_feed::delete(params.id, params.profile_id)
                .build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&self.pool)
                .await
                .map(|e| e.rows_affected())
                .map_err(|e| Error::Unknown(e.into()))?
        };
        if count == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
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

#[derive(Debug, Clone, sqlx::FromRow)]
struct SmartFeedSelect {
    id: Uuid,
    title: String,
    unread_count: i64,
}

impl From<SmartFeedSelect> for colette_core::SmartFeed {
    fn from(value: SmartFeedSelect) -> Self {
        Self {
            id: value.id,
            title: value.title,
            unread_count: Some(value.unread_count),
        }
    }
}
pub(crate) async fn find(
    executor: impl PgExecutor<'_>,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
) -> Result<Vec<SmartFeed>, Error> {
    let (sql, values) =
        colette_sql::smart_feed::select(id, profile_id, cursor, limit, build_case_statement())
            .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with::<_, SmartFeedSelect, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(SmartFeed::from).collect::<Vec<_>>())
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
    filters: Vec<Filter>,
    smart_feed_id: Uuid,
    profile_id: Uuid,
) -> sqlx::Result<()> {
    let insert_data = filters
        .into_iter()
        .map(|e| {
            let (field, op): (Field, Op) = match e {
                Filter::Link(op) => (Field::Link, op.into()),
                Filter::Title(op) => (Field::Title, op.into()),
                Filter::PublishedAt(op) => (Field::PublishedAt, op.into()),
                Filter::Description(op) => (Field::Description, op.into()),
                Filter::Author(op) => (Field::Author, op.into()),
                Filter::HasRead(op) => (
                    Field::HasRead,
                    Op {
                        r#type: Operation::Eq,
                        value: op.value.to_string(),
                    },
                ),
            };

            colette_sql::smart_feed_filter::InsertMany {
                id: None,
                field,
                operation: op.r#type,
                value: op.value,
            }
        })
        .collect::<Vec<_>>();

    {
        let (sql, values) =
            colette_sql::smart_feed_filter::insert_many(insert_data, smart_feed_id, profile_id)
                .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(executor).await?;
    }

    Ok(())
}

pub(crate) trait SmartFilterCase {
    fn add_smart_filter(self, field: Field, operation: Operation) -> Self;
}

impl SmartFilterCase for CaseStatement {
    fn add_smart_filter(self, field: Field, operation: Operation) -> Self {
        let value_col = Expr::col((SmartFeedFilter::Table, SmartFeedFilter::Value));

        let field_col: SimpleExpr = match field {
            Field::Link => Expr::col((FeedEntry::Table, FeedEntry::Link)).into(),
            Field::Title => Expr::col((FeedEntry::Table, FeedEntry::Title)).into(),
            Field::PublishedAt => Func::cast_as(
                Expr::col((FeedEntry::Table, FeedEntry::PublishedAt)),
                Alias::new("text"),
            )
            .into(),
            Field::Description => Expr::col((FeedEntry::Table, FeedEntry::Description)).into(),
            Field::Author => Expr::col((FeedEntry::Table, FeedEntry::Author)).into(),
            Field::HasRead => Func::cast_as(
                Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::HasRead)),
                Alias::new("text"),
            )
            .into(),
            Field::Type => unreachable!(),
        };

        let cond = Expr::col((SmartFeedFilter::Table, SmartFeedFilter::Field))
            .eq(Func::cast_as(
                Expr::val(field.to_string()),
                Alias::new("field"),
            ))
            .and(
                Expr::col((SmartFeedFilter::Table, SmartFeedFilter::Operation)).eq(Func::cast_as(
                    Expr::val(operation.to_string()),
                    Alias::new("operation"),
                )),
            );

        let then = match operation {
            Operation::Eq => field_col.eq(value_col),
            Operation::Ne => field_col.ne(value_col),
            Operation::Like => {
                Expr::cust_with_exprs("$1 LIKE '%' || $2 ||'%'", [field_col, value_col.into()])
            }
            Operation::NotLike => Expr::cust_with_exprs(
                "$1 NOT LIKE '%' || $2 || '%'",
                [field_col, value_col.into()],
            ),
            Operation::GreaterThan => Expr::expr(field_col).gt(value_col),
            Operation::LessThan => Expr::expr(field_col).lt(value_col),
            Operation::InLastXSec => Expr::cust_with_exprs(
                "EXTRACT(EPOCH FROM ($1 - $2)) < $3",
                [
                    Expr::current_timestamp().into(),
                    Func::cast_as(field_col, Alias::new("timestamptz")).into(),
                    Func::cast_as(value_col, Alias::new("numeric")).into(),
                ],
            ),
            Operation::Type => unreachable!(),
        };

        self.case(cond, then)
    }
}

pub(crate) fn build_case_statement() -> CaseStatement {
    CaseStatement::new()
        .add_smart_filter(Field::Link, Operation::Eq)
        .add_smart_filter(Field::Link, Operation::Ne)
        .add_smart_filter(Field::Link, Operation::Like)
        .add_smart_filter(Field::Link, Operation::NotLike)
        .add_smart_filter(Field::Title, Operation::Eq)
        .add_smart_filter(Field::Title, Operation::Ne)
        .add_smart_filter(Field::Title, Operation::Like)
        .add_smart_filter(Field::Title, Operation::NotLike)
        .add_smart_filter(Field::PublishedAt, Operation::Eq)
        .add_smart_filter(Field::PublishedAt, Operation::Ne)
        .add_smart_filter(Field::PublishedAt, Operation::GreaterThan)
        .add_smart_filter(Field::PublishedAt, Operation::LessThan)
        .add_smart_filter(Field::PublishedAt, Operation::InLastXSec)
        .add_smart_filter(Field::Description, Operation::Eq)
        .add_smart_filter(Field::Description, Operation::Ne)
        .add_smart_filter(Field::Description, Operation::Like)
        .add_smart_filter(Field::Description, Operation::NotLike)
        .add_smart_filter(Field::Author, Operation::Eq)
        .add_smart_filter(Field::Author, Operation::Ne)
        .add_smart_filter(Field::Author, Operation::Like)
        .add_smart_filter(Field::Author, Operation::NotLike)
        .add_smart_filter(Field::HasRead, Operation::Eq)
}
