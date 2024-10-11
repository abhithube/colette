use sea_query::{
    ColumnDef, ColumnType, DeleteStatement, Expr, InsertStatement, OnConflict, Query,
    SelectStatement, Table, TableCreateStatement,
};

use crate::{common::WithTimestamps, profile_feed::ProfileFeed};

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

pub fn create_table(timestamp_type: ColumnType) -> TableCreateStatement {
    Table::create()
        .table(Feed::Table)
        .if_not_exists()
        .col(
            ColumnDef::new_with_type(Feed::Id, ColumnType::Integer)
                .not_null()
                .primary_key()
                .auto_increment(),
        )
        .col(
            ColumnDef::new_with_type(Feed::Link, ColumnType::Text)
                .not_null()
                .unique_key(),
        )
        .col(ColumnDef::new_with_type(Feed::Title, ColumnType::Text).not_null())
        .col(ColumnDef::new_with_type(Feed::Url, ColumnType::Text))
        .with_timestamps(timestamp_type)
        .to_owned()
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
