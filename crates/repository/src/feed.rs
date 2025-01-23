use std::fmt::Write;

use sea_query::{Alias, Expr, Func, Iden, InsertStatement, OnConflict, Query, SelectStatement};

use crate::user_feed::UserFeed;

#[allow(dead_code)]
pub enum Feed {
    Table,
    Id,
    Link,
    Title,
    XmlUrl,
    CreatedAt,
    UpdatedAt,
}

impl Iden for Feed {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "feeds",
                Self::Id => "id",
                Self::Link => "link",
                Self::Title => "title",
                Self::XmlUrl => "xml_url",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

pub fn select_by_url(url: String) -> SelectStatement {
    Query::select()
        .column(Feed::Id)
        .from(Feed::Table)
        .and_where(
            Expr::col(Feed::XmlUrl)
                .eq(url.clone())
                .or(Expr::col(Feed::Link).eq(url)),
        )
        .to_owned()
}

pub fn insert(link: String, title: String, xml_url: Option<String>) -> InsertStatement {
    Query::insert()
        .into_table(Feed::Table)
        .columns([Feed::Link, Feed::Title, Feed::XmlUrl, Feed::UpdatedAt])
        .values_panic([
            link.into(),
            title.into(),
            xml_url.into(),
            Expr::current_timestamp().into(),
        ])
        .on_conflict(
            OnConflict::column(Feed::Link)
                .update_columns([Feed::Title, Feed::XmlUrl, Feed::UpdatedAt])
                .to_owned(),
        )
        .returning_col(Feed::Id)
        .to_owned()
}

pub fn iterate() -> SelectStatement {
    Query::select()
        .expr_as(
            Func::coalesce([
                Expr::col((Feed::Table, Feed::XmlUrl)).into(),
                Expr::col((Feed::Table, Feed::Link)).into(),
            ]),
            Alias::new("url"),
        )
        .from(Feed::Table)
        .inner_join(
            UserFeed::Table,
            Expr::col((UserFeed::Table, UserFeed::FeedId)).eq(Expr::col((Feed::Table, Feed::Id))),
        )
        .to_owned()
}
