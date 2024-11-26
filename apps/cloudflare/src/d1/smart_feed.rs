use std::sync::Arc;

use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    smart_feed::{
        DateOperation, Error, SmartFeedCreateData, SmartFeedFilter as Filter, SmartFeedFindParams,
        SmartFeedRepository, SmartFeedUpdateData, TextOperation,
    },
    SmartFeed,
};
use colette_sql::{
    feed_entry::FeedEntry,
    profile_feed_entry::ProfileFeedEntry,
    smart_feed_filter::{Field, Operation, SmartFeedFilter},
};
use sea_query::{Alias, CaseStatement, Expr, Func, Iden, SimpleExpr, SqliteQueryBuilder};
use uuid::Uuid;
use worker::D1Database;

use super::{D1Binder, D1Error, D1Values};

#[derive(Clone)]
pub struct D1SmartFeedRepository {
    db: Arc<D1Database>,
}

impl D1SmartFeedRepository {
    pub fn new(db: Arc<D1Database>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for D1SmartFeedRepository {
    type Params = SmartFeedFindParams;
    type Output = Result<Vec<SmartFeed>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let (sql, values) = colette_sql::smart_feed::select(
            params.id,
            params.profile_id,
            params.cursor,
            params.limit,
            build_case_statement(),
        )
        .build_d1(SqliteQueryBuilder);

        let result = super::all(&self.db, sql, values)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        result
            .results::<SmartFeed>()
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for D1SmartFeedRepository {
    type Data = SmartFeedCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let id = Uuid::new_v4();

        let mut queries =
            vec![
                colette_sql::smart_feed::insert(Some(id), data.title.clone(), data.profile_id)
                    .build_d1(SqliteQueryBuilder),
            ];

        if let Some(filters) = data.filters {
            queries.push(build_insert_filters_query(filters, id, data.profile_id))
        }

        super::batch(&self.db, queries)
            .await
            .map_err(|e| match e.into() {
                D1Error::UniqueConstraint => Error::Conflict(data.title),
                e => Error::Unknown(e.into()),
            })?;

        Ok(id)
    }
}

#[async_trait::async_trait]
impl Updatable for D1SmartFeedRepository {
    type Params = IdParams;
    type Data = SmartFeedUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut queries: Vec<(String, D1Values)> = Vec::new();

        if data.title.is_some() {
            queries.push(
                colette_sql::smart_feed::update(params.id, params.profile_id, data.title)
                    .build_d1(SqliteQueryBuilder),
            );
        }

        if let Some(filters) = data.filters {
            queries.push(
                colette_sql::smart_feed_filter::delete_many(params.profile_id, params.id)
                    .build_d1(SqliteQueryBuilder),
            );

            queries.push(build_insert_filters_query(
                filters,
                params.id,
                params.profile_id,
            ))
        }

        if !queries.is_empty() {
            let results = super::batch(&self.db, queries)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            if let Some(first) = results.first() {
                let meta = first.meta().unwrap().unwrap();

                if meta.changes.is_none_or(|e| e == 0) {
                    return Err(Error::NotFound(params.id));
                }
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for D1SmartFeedRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let (sql, values) = colette_sql::smart_feed::delete(params.id, params.profile_id)
            .build_d1(SqliteQueryBuilder);

        let result = super::run(&self.db, sql, values)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let meta = result.meta().unwrap().unwrap();

        if meta.changes.is_none_or(|e| e == 0) {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

impl SmartFeedRepository for D1SmartFeedRepository {}

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

fn build_insert_filters_query(
    filters: Vec<Filter>,
    smart_feed_id: Uuid,
    profile_id: Uuid,
) -> (String, D1Values) {
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
                id: Some(Uuid::new_v4()),
                field,
                operation: op.r#type,
                value: op.value,
            }
        })
        .collect::<Vec<_>>();

    colette_sql::smart_feed_filter::insert_many(&insert_data, smart_feed_id, profile_id)
        .build_d1(SqliteQueryBuilder)
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
