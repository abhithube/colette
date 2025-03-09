use std::fmt::Write;

use chrono::{DateTime, Utc};
use colette_core::feed::{FeedFindParams, FeedStreamUrlsParams};
use sea_query::{
    Asterisk, Expr, Func, Iden, InsertStatement, OnConflict, Order, Query, SelectStatement,
};
use url::Url;
use uuid::Uuid;

use crate::{IntoInsert, IntoSelect};

pub enum Feed {
    Table,
    Id,
    Link,
    XmlUrl,
    Title,
    Description,
    RefreshedAt,
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
                Self::XmlUrl => "xml_url",
                Self::Title => "title",
                Self::Description => "description",
                Self::RefreshedAt => "refreshed_at",
            }
        )
        .unwrap();
    }
}

impl IntoSelect for FeedFindParams {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .column(Asterisk)
            .from(Feed::Table)
            .apply_if(self.id, |query, id| {
                query.and_where(Expr::col((Feed::Table, Feed::Id)).eq(id.to_string()));
            })
            .apply_if(self.cursor, |query, cursor| {
                query.and_where(
                    Expr::col((Feed::Table, Feed::Link)).gt(Expr::val(cursor.link.to_string())),
                );
            })
            .order_by((Feed::Table, Feed::Link), Order::Asc)
            .to_owned();

        if let Some(limit) = self.limit {
            query.limit(limit as u64);
        }

        query
    }
}

pub struct FeedUpsert {
    pub id: Uuid,
    pub link: Url,
    pub xml_url: Option<Url>,
    pub title: String,
    pub description: Option<String>,
    pub refreshed_at: Option<DateTime<Utc>>,
}

impl IntoInsert for FeedUpsert {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(Feed::Table)
            .columns([
                Feed::Id,
                Feed::Link,
                Feed::XmlUrl,
                Feed::Title,
                Feed::Description,
                Feed::RefreshedAt,
            ])
            .values_panic([
                self.id.to_string().into(),
                self.link.to_string().into(),
                self.xml_url.map(String::from).into(),
                self.title.into(),
                self.description.into(),
                self.refreshed_at.map(|e| e.timestamp()).into(),
            ])
            .on_conflict(
                OnConflict::column(Feed::Link)
                    .update_columns([
                        Feed::XmlUrl,
                        Feed::Title,
                        Feed::Description,
                        Feed::RefreshedAt,
                    ])
                    .to_owned(),
            )
            .returning_col(Feed::Id)
            .to_owned()
    }
}

impl IntoSelect for FeedStreamUrlsParams {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .expr(Func::coalesce([
                Expr::col(Feed::XmlUrl).into(),
                Expr::col(Feed::Link).into(),
            ]))
            .from(Feed::Table)
            .to_owned()
    }
}
