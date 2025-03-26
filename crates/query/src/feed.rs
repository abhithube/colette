use std::fmt::Write;

use chrono::{DateTime, Utc};
use colette_core::feed::FeedParams;
use sea_query::{Asterisk, Expr, Iden, InsertStatement, OnConflict, Order, Query, SelectStatement};
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

impl IntoSelect for FeedParams {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .column(Asterisk)
            .from(Feed::Table)
            .apply_if(self.id, |query, id| {
                query.and_where(Expr::col((Feed::Table, Feed::Id)).eq(id));
            })
            .apply_if(self.cursor, |query, link| {
                query.and_where(Expr::col((Feed::Table, Feed::Link)).gt(Expr::val(link)));
            })
            .order_by((Feed::Table, Feed::Link), Order::Asc)
            .to_owned();

        if let Some(limit) = self.limit {
            query.limit(limit);
        }

        query
    }
}

pub struct FeedInsert<'a> {
    pub id: Uuid,
    pub link: &'a str,
    pub xml_url: Option<&'a str>,
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub refreshed_at: Option<DateTime<Utc>>,
}

impl IntoInsert for FeedInsert<'_> {
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
                self.id.into(),
                self.link.into(),
                self.xml_url.into(),
                self.title.into(),
                self.description.into(),
                self.refreshed_at.into(),
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
