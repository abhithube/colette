use std::fmt::Write;

use sea_query::{DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Query, SelectStatement};

use crate::profile_feed::ProfileFeed;

pub enum Feed {
    Table,
    Id,
    Link,
    Title,
    Url,
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
                Self::Url => "url",
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
            Expr::col(Feed::Url)
                .eq(url.clone())
                .or(Expr::col(Feed::Link).eq(url)),
        )
        .to_owned()
}

pub fn insert(link: String, title: String, url: Option<String>) -> InsertStatement {
    Query::insert()
        .into_table(Feed::Table)
        .columns([Feed::Link, Feed::Title, Feed::Url, Feed::UpdatedAt])
        .values_panic([
            link.into(),
            title.into(),
            url.into(),
            Expr::current_timestamp().into(),
        ])
        .on_conflict(
            OnConflict::column(Feed::Link)
                .update_columns([Feed::Title, Feed::Url, Feed::UpdatedAt])
                .to_owned(),
        )
        .returning_col(Feed::Id)
        .to_owned()
}

pub fn delete_many() -> DeleteStatement {
    let subquery = Query::select()
        .from(ProfileFeed::Table)
        .and_where(
            Expr::col((ProfileFeed::Table, ProfileFeed::FeedId)).equals((Feed::Table, Feed::Id)),
        )
        .to_owned();

    Query::delete()
        .from_table(Feed::Table)
        .and_where(Expr::exists(subquery).not())
        .to_owned()
}
