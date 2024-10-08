use sea_query::{
    DeleteStatement, Expr, InsertStatement, OnConflict, Query, SelectStatement, UpdateStatement,
};
use sqlx::types::Uuid;

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub(crate) enum ProfileFeed {
    Table,
    Id,
    Title,
    Pinned,
    ProfileId,
    FeedId,
    CreatedAt,
    UpdatedAt,
}

pub fn select_by_unique_index(profile_id: Uuid, feed_id: i32) -> SelectStatement {
    Query::select()
        .column(ProfileFeed::Id)
        .from(ProfileFeed::Table)
        .and_where(Expr::col(ProfileFeed::ProfileId).eq(profile_id))
        .and_where(Expr::col(ProfileFeed::FeedId).eq(feed_id))
        .to_owned()
}

pub fn insert(id: Uuid, pinned: Option<bool>, feed_id: i32, profile_id: Uuid) -> InsertStatement {
    Query::insert()
        .into_table(ProfileFeed::Table)
        .columns([
            ProfileFeed::Id,
            ProfileFeed::Pinned,
            ProfileFeed::FeedId,
            ProfileFeed::ProfileId,
        ])
        .values_panic([
            id.into(),
            pinned.unwrap_or_default().into(),
            feed_id.into(),
            profile_id.into(),
        ])
        .on_conflict(
            OnConflict::columns([ProfileFeed::ProfileId, ProfileFeed::FeedId])
                .do_nothing()
                .to_owned(),
        )
        .returning_col(ProfileFeed::Id)
        .to_owned()
}

pub fn update(
    id: Uuid,
    profile_id: Uuid,
    title: Option<Option<String>>,
    pinned: Option<bool>,
) -> UpdateStatement {
    let mut query = Query::update()
        .table(ProfileFeed::Table)
        .and_where(Expr::col((ProfileFeed::Table, ProfileFeed::Id)).eq(id))
        .and_where(Expr::col((ProfileFeed::Table, ProfileFeed::ProfileId)).eq(profile_id))
        .to_owned();

    if let Some(title) = title {
        query.value(ProfileFeed::Title, title);
    }
    if let Some(pinned) = pinned {
        query.value(ProfileFeed::Pinned, pinned);
    }

    query
}

pub fn delete(id: Uuid, profile_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(ProfileFeed::Table)
        .and_where(Expr::col((ProfileFeed::Table, ProfileFeed::Id)).eq(id))
        .and_where(Expr::col((ProfileFeed::Table, ProfileFeed::ProfileId)).eq(profile_id))
        .to_owned()
}
