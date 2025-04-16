use std::fmt::Write;

use chrono::{DateTime, Utc};
use colette_core::feed::FeedParams;
use sea_query::{Asterisk, Expr, Iden, InsertStatement, OnConflict, Order, Query, SelectStatement};
use uuid::Uuid;

use crate::{IntoInsert, IntoSelect};

pub enum Feed {
    Table,
    Id,
    SourceUrl,
    Link,
    Title,
    Description,
    RefreshedAt,
    IsCustom,
}

impl Iden for Feed {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "feeds",
                Self::Id => "id",
                Self::SourceUrl => "source_url",
                Self::Link => "link",
                Self::Title => "title",
                Self::Description => "description",
                Self::RefreshedAt => "refreshed_at",
                Self::IsCustom => "is_custom",
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
            .apply_if(self.source_url, |query, source_url| {
                query.and_where(Expr::col((Feed::Table, Feed::SourceUrl)).eq(source_url.as_str()));
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
    pub source_url: &'a str,
    pub link: &'a str,
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub refreshed_at: Option<DateTime<Utc>>,
    pub is_custom: bool,
}

impl IntoInsert for FeedInsert<'_> {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(Feed::Table)
            .columns([
                Feed::Id,
                Feed::SourceUrl,
                Feed::Link,
                Feed::Title,
                Feed::Description,
                Feed::RefreshedAt,
                Feed::IsCustom,
            ])
            .values_panic([
                self.id.into(),
                self.source_url.into(),
                self.link.into(),
                self.title.into(),
                self.description.into(),
                self.refreshed_at.into(),
                self.is_custom.into(),
            ])
            .on_conflict(
                OnConflict::column(Feed::SourceUrl)
                    .update_columns([
                        Feed::Link,
                        Feed::Title,
                        Feed::Description,
                        Feed::RefreshedAt,
                        Feed::IsCustom,
                    ])
                    .to_owned(),
            )
            .returning_col(Feed::Id)
            .to_owned()
    }
}
