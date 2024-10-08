use sea_query::{DeleteStatement, Expr, InsertStatement, OnConflict, Query, SelectStatement};
use sqlx::types::Uuid;

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub(crate) enum ProfileBookmark {
    Table,
    Id,
    ProfileId,
    BookmarkId,
    CreatedAt,
    UpdatedAt,
}

pub fn select_by_unique_index(profile_id: Uuid, bookmark_id: i32) -> SelectStatement {
    Query::select()
        .column(ProfileBookmark::Id)
        .from(ProfileBookmark::Table)
        .and_where(Expr::col(ProfileBookmark::ProfileId).eq(profile_id))
        .and_where(Expr::col(ProfileBookmark::BookmarkId).eq(bookmark_id))
        .to_owned()
}

pub fn insert(id: Uuid, bookmark_id: i32, profile_id: Uuid) -> InsertStatement {
    Query::insert()
        .into_table(ProfileBookmark::Table)
        .columns([
            ProfileBookmark::Id,
            ProfileBookmark::BookmarkId,
            ProfileBookmark::ProfileId,
        ])
        .values_panic([id.into(), bookmark_id.into(), profile_id.into()])
        .on_conflict(
            OnConflict::columns([ProfileBookmark::ProfileId, ProfileBookmark::BookmarkId])
                .do_nothing()
                .to_owned(),
        )
        .returning_col(ProfileBookmark::Id)
        .to_owned()
}

pub fn delete(id: Uuid, profile_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(ProfileBookmark::Table)
        .and_where(Expr::col((ProfileBookmark::Table, ProfileBookmark::Id)).eq(id))
        .and_where(Expr::col((ProfileBookmark::Table, ProfileBookmark::ProfileId)).eq(profile_id))
        .to_owned()
}
