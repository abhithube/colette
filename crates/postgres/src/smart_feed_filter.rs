use std::fmt;

use sea_query::{Alias, CaseStatement, Expr, Func, SimpleExpr};

use crate::{feed_entry::FeedEntry, profile_feed_entry::ProfileFeedEntry};

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub(crate) enum SmartFeedFilter {
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

pub(crate) enum Field {
    Link,
    Title,
    PublishedAt,
    Description,
    Author,
    HasRead,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::Link => "link",
            Self::Title => "title",
            Self::PublishedAt => "published_at",
            Self::Description => "description",
            Self::Author => "author",
            Self::HasRead => "has_read",
        };

        write!(f, "{}", str)
    }
}

pub(crate) enum Operation {
    Eq,
    Ne,
    Like,
    NotLike,
    GreaterThan,
    LessThan,
    InLastXSec,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::Eq => "=",
            Self::Ne => "!=",
            Self::Like => "LIKE",
            Self::NotLike => "NOT LIKE",
            Self::GreaterThan => ">",
            Self::LessThan => "<",
            Self::InLastXSec => "in_last_x_sec",
        };

        write!(f, "{}", str)
    }
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
                "EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - $1)) < $2",
                [
                    Func::cast_as(field_col, Alias::new("timestamptz")).into(),
                    Func::cast_as(value_col, Alias::new("numeric")).into(),
                ],
            ),
        };

        self.case(cond, then)
    }
}
