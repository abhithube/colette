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
use deadpool_sqlite::Pool;
use rusqlite::{Connection, Row};
use sea_query::{Alias, CaseStatement, Expr, Func, Iden, SimpleExpr, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

pub struct SqliteSmartFeedRepository {
    pool: Pool,
}

impl SqliteSmartFeedRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteSmartFeedRepository {
    type Params = IdParams;
    type Output = Result<SmartFeed, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| find_by_id(conn, params.id, params.profile_id))
            .await
            .unwrap()
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteSmartFeedRepository {
    type Data = SmartFeedCreateData;
    type Output = Result<SmartFeed, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let title = data.title.clone();

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            let id = Uuid::new_v4();

            {
                let (sql, values) =
                    colette_sql::smart_feed::insert(id, data.title.clone(), data.profile_id)
                        .build_rusqlite(SqliteQueryBuilder);

                tx.prepare_cached(&sql)?.execute(&*values.as_params())?;
            }

            if let Some(filters) = data.filters {
                insert_filters(&tx, filters, id, data.profile_id)?;
            }

            let feed = find_by_id(&tx, id, data.profile_id)?;

            tx.commit()?;

            Ok(feed)
        })
        .await
        .unwrap()
        .map_err(|e| match e {
            rusqlite::Error::SqliteFailure(e, _)
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Error::Conflict(title)
            }
            _ => Error::Unknown(e.into()),
        })
    }
}

#[async_trait::async_trait]
impl Updatable for SqliteSmartFeedRepository {
    type Params = IdParams;
    type Data = SmartFeedUpdateData;
    type Output = Result<SmartFeed, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            if data.title.is_some() {
                let (sql, values) =
                    colette_sql::smart_feed::update(params.id, params.profile_id, data.title)
                        .build_rusqlite(SqliteQueryBuilder);

                let count = tx.prepare_cached(&sql)?.execute(&*values.as_params())?;
                if count == 0 {
                    return Err(rusqlite::Error::QueryReturnedNoRows);
                }
            }

            if let Some(filters) = data.filters {
                {
                    let (sql, values) =
                        colette_sql::smart_feed_filter::delete_many(params.profile_id, params.id)
                            .build_rusqlite(SqliteQueryBuilder);

                    tx.prepare_cached(&sql)?.execute(&*values.as_params())?;
                }

                insert_filters(&tx, filters, params.id, params.profile_id)?;
            }

            let feed = find_by_id(&tx, params.id, params.profile_id)?;

            tx.commit()?;

            Ok(feed)
        })
        .await
        .unwrap()
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }
}

#[async_trait::async_trait]
impl Deletable for SqliteSmartFeedRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let (sql, values) = colette_sql::smart_feed::delete(params.id, params.profile_id)
                .build_rusqlite(SqliteQueryBuilder);

            let count = conn.prepare_cached(&sql)?.execute(&*values.as_params())?;
            if count == 0 {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }

            Ok(())
        })
        .await
        .unwrap()
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }
}

#[async_trait::async_trait]
impl SmartFeedRepository for SqliteSmartFeedRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
    ) -> Result<Vec<SmartFeed>, Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| find(conn, None, profile_id, limit, cursor))
            .await
            .unwrap()
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[derive(Debug, Clone)]
struct SmartFeedSelect(SmartFeed);

impl TryFrom<&Row<'_>> for SmartFeedSelect {
    type Error = rusqlite::Error;

    fn try_from(value: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self(SmartFeed {
            id: value.get("id")?,
            title: value.get("title")?,
            unread_count: Some(value.get("unread_count")?),
        }))
    }
}

pub(crate) fn find(
    conn: &Connection,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
) -> rusqlite::Result<Vec<SmartFeed>> {
    let (sql, values) =
        colette_sql::smart_feed::select(id, profile_id, cursor, limit, build_case_statement())
            .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = conn.prepare_cached(&sql)?;
    let mut rows = stmt.query(&*values.as_params())?;

    let mut feeds: Vec<SmartFeed> = Vec::new();
    while let Some(row) = rows.next()? {
        feeds.push(SmartFeedSelect::try_from(row).map(|e| e.0)?);
    }

    Ok(feeds)
}

fn find_by_id(conn: &Connection, id: Uuid, profile_id: Uuid) -> rusqlite::Result<SmartFeed> {
    let mut feeds = find(conn, Some(id), profile_id, None, None)?;
    if feeds.is_empty() {
        return Err(rusqlite::Error::QueryReturnedNoRows);
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

fn insert_filters(
    conn: &Connection,
    filters: Vec<Filter>,
    smart_feed_id: Uuid,
    profile_id: Uuid,
) -> rusqlite::Result<()> {
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
                id: Uuid::new_v4(),
                field,
                operation: op.r#type,
                value: op.value,
            }
        })
        .collect::<Vec<_>>();

    {
        let (sql, values) =
            colette_sql::smart_feed_filter::insert_many(insert_data, smart_feed_id, profile_id)
                .build_rusqlite(SqliteQueryBuilder);

        conn.prepare_cached(&sql)?.execute(&*values.as_params())?;
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
            Field::PublishedAt => Expr::col((FeedEntry::Table, FeedEntry::PublishedAt)).into(),
            Field::Description => Expr::col((FeedEntry::Table, FeedEntry::Description)).into(),
            Field::Author => Expr::col((FeedEntry::Table, FeedEntry::Author)).into(),
            Field::HasRead => Func::cast_as(
                Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::HasRead)),
                Alias::new("TEXT"),
            )
            .into(),
            Field::Type => unreachable!(),
        };

        let cond = Expr::col((SmartFeedFilter::Table, SmartFeedFilter::Field))
            .eq(Expr::val(field.to_string()))
            .and(
                Expr::col((SmartFeedFilter::Table, SmartFeedFilter::Operation))
                    .eq(Expr::val(operation.to_string())),
            );

        let then = match operation {
            Operation::Eq => field_col.eq(value_col),
            Operation::Ne => field_col.ne(value_col),
            Operation::Like => {
                Expr::cust_with_exprs("? LIKE '%' || ? ||'%'", [field_col, value_col.into()])
            }
            Operation::NotLike => {
                Expr::cust_with_exprs("? NOT LIKE '%' || ? || '%'", [field_col, value_col.into()])
            }
            Operation::GreaterThan => Expr::expr(field_col).gt(value_col),
            Operation::LessThan => Expr::expr(field_col).lt(value_col),
            Operation::InLastXSec => Expr::cust_with_exprs(
                "STRFTIME('%s', ?) - STRFTIME('%s', ?) < ?",
                [
                    Expr::current_timestamp().into(),
                    Func::cast_as(field_col, Alias::new("TEXT")).into(),
                    Func::cast_as(value_col, Alias::new("NUMERIC")).into(),
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
