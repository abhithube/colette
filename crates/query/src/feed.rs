use std::fmt::Write;

use chrono::{DateTime, Utc};
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

#[derive(Default)]
pub struct FeedSelect<'a> {
    pub id: Option<Uuid>,
    pub source_urls: Option<Vec<&'a str>>,
    pub cursor: Option<&'a str>,
    pub limit: Option<u64>,
}

impl IntoSelect for FeedSelect<'_> {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .column(Asterisk)
            .from(Feed::Table)
            .apply_if(self.id, |query, id| {
                query.and_where(Expr::col((Feed::Table, Feed::Id)).eq(id));
            })
            .apply_if(self.source_urls, |query, source_urls| {
                query.and_where(Expr::col((Feed::Table, Feed::SourceUrl)).is_in(source_urls));
            })
            .apply_if(self.cursor, |query, source_url| {
                query
                    .and_where(Expr::col((Feed::Table, Feed::SourceUrl)).gt(Expr::val(source_url)));
            })
            .order_by((Feed::Table, Feed::SourceUrl), Order::Asc)
            .to_owned();

        if let Some(limit) = self.limit {
            query.limit(limit);
        }

        query
    }
}

pub struct FeedInsert<I> {
    pub feeds: I,
    pub upsert: bool,
}

pub struct FeedBase<'a> {
    pub id: Uuid,
    pub source_url: &'a str,
    pub link: &'a str,
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub refreshed_at: Option<DateTime<Utc>>,
    pub is_custom: bool,
}

impl<'a, I: IntoIterator<Item = FeedBase<'a>>> IntoInsert for FeedInsert<I> {
    fn into_insert(self) -> InsertStatement {
        let mut query = Query::insert()
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
            .returning_col(Feed::Id)
            .to_owned();

        if self.upsert {
            query.on_conflict(
                OnConflict::column(Feed::SourceUrl)
                    .update_columns([
                        Feed::Link,
                        Feed::Title,
                        Feed::Description,
                        Feed::RefreshedAt,
                        Feed::IsCustom,
                    ])
                    .to_owned(),
            );
        } else {
            query.on_conflict(OnConflict::column(Feed::SourceUrl).do_nothing().to_owned());
        }

        for feed in self.feeds.into_iter() {
            query.values_panic([
                feed.id.into(),
                feed.source_url.into(),
                feed.link.into(),
                feed.title.into(),
                feed.description.into(),
                feed.refreshed_at.into(),
                feed.is_custom.into(),
            ]);
        }

        query
    }
}
