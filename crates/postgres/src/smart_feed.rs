use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    smart_feed::{
        Cursor, DateOperation, Error, SmartFeedCreateData, SmartFeedFilter as Filter,
        SmartFeedRepository, SmartFeedUpdateData, TextOperation,
    },
    SmartFeed,
};
use colette_sql::smart_feed_filter::{Field, Operation};
use deadpool_postgres::{GenericClient, Pool};
use sea_query::{Alias, CaseStatement, Expr, Func, PostgresQueryBuilder, SimpleExpr};
use sea_query_postgres::PostgresBinder;
use tokio_postgres::{error::SqlState, Row};
use uuid::Uuid;

pub struct PostgresSmartFeedRepository {
    pub(crate) pool: Pool,
}

impl PostgresSmartFeedRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresSmartFeedRepository {
    type Params = IdParams;
    type Output = Result<SmartFeed, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        find_by_id(&client, params).await
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresSmartFeedRepository {
    type Data = SmartFeedCreateData;
    type Output = Result<SmartFeed, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let id = Uuid::new_v4();

        {
            let (sql, values) =
                colette_sql::smart_feed::insert(id, data.title.clone(), data.profile_id)
                    .build_postgres(PostgresQueryBuilder);

            tx.execute(&sql, &values.as_params())
                .await
                .map_err(|e| match e.code() {
                    Some(&SqlState::UNIQUE_VIOLATION) => Error::Conflict(data.title),
                    _ => Error::Unknown(e.into()),
                })?;
        };

        if let Some(filters) = data.filters {
            insert_filters(&tx, filters, id, data.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        let feed = find_by_id(&tx, IdParams::new(id, data.profile_id)).await?;

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
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.title.is_some() {
            let count = {
                let (sql, values) =
                    colette_sql::smart_feed::update(params.id, params.profile_id, data.title)
                        .build_postgres(PostgresQueryBuilder);

                tx.execute(&sql, &values.as_params())
                    .await
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
                        .build_postgres(PostgresQueryBuilder);

                tx.execute(&sql, &values.as_params())
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;
            }

            insert_filters(&tx, filters, params.id, params.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        let feed = find_by_id(&tx, params)
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
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let count = {
            let (sql, values) = colette_sql::smart_feed::delete(params.id, params.profile_id)
                .build_postgres(PostgresQueryBuilder);

            client
                .execute(&sql, &values.as_params())
                .await
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
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        find(&client, None, profile_id, limit, cursor).await
    }
}

#[derive(Debug, Clone)]
struct SmartFeedSelect(SmartFeed);

impl From<&Row> for SmartFeedSelect {
    fn from(value: &Row) -> Self {
        Self(SmartFeed {
            id: value.get("id"),
            title: value.get("title"),
            unread_count: Some(value.get("unread_count")),
        })
    }
}

pub(crate) async fn find<C: GenericClient>(
    client: &C,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
) -> Result<Vec<SmartFeed>, Error> {
    let (sql, values) =
        colette_sql::smart_feed::select(id, profile_id, cursor, limit, build_case_statement())
            .build_postgres(PostgresQueryBuilder);

    client
        .query(&sql, &values.as_params())
        .await
        .map(|e| {
            e.into_iter()
                .map(|e| SmartFeedSelect::from(&e).0)
                .collect::<Vec<_>>()
        })
        .map_err(|e| Error::Unknown(e.into()))
}

async fn find_by_id<C: GenericClient>(client: &C, params: IdParams) -> Result<SmartFeed, Error> {
    let mut feeds = find(client, Some(params.id), params.profile_id, None, None).await?;
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

async fn insert_filters<C: GenericClient>(
    client: &C,
    filters: Vec<Filter>,
    smart_feed_id: Uuid,
    profile_id: Uuid,
) -> Result<(), tokio_postgres::Error> {
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
                .build_postgres(PostgresQueryBuilder);

        client.execute(&sql, &values.as_params()).await?;
    }

    Ok(())
}

#[allow(dead_code)]
#[derive(sea_query::Iden)]
enum FeedEntry {
    Table,
    Id,
    Link,
    Title,
    PublishedAt,
    Description,
    Author,
    ThumbnailUrl,
    FeedId,
    CreatedAt,
    UpdatedAt,
}

#[allow(dead_code)]
#[derive(sea_query::Iden)]
enum ProfileFeedEntry {
    Table,
    Id,
    HasRead,
    ProfileFeedId,
    FeedEntryId,
    ProfileId,
    CreatedAt,
    UpdatedAt,
}

#[allow(dead_code)]
#[derive(sea_query::Iden)]
enum SmartFeedFilter {
    Table,
    Id,
    Field,
    Operation,
    Value,
    SmartFeedId,
    ProfileId,
    CreatedAt,
    UpdatedAt,
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
