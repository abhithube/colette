use sea_query::{DeleteStatement, Expr, InsertStatement, OnConflict, Query, SelectStatement};

use crate::profile_feed::ProfileFeed;

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub enum Feed {
    Table,
    Id,
    Link,
    Title,
    Url,
    CreatedAt,
    UpdatedAt,
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
