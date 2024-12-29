use std::fmt::Write;

use sea_query::{DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Query};
use uuid::Uuid;

use crate::tag::build_titles_subquery;

#[allow(dead_code)]
pub enum UserBookmarkTag {
    Table,
    UserBookmarkId,
    TagId,
    UserId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for UserBookmarkTag {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "user_bookmark_tags",
                Self::UserBookmarkId => "user_bookmark_id",
                Self::TagId => "tag_id",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

pub struct InsertMany {
    pub user_bookmark_id: Uuid,
    pub tag_id: Uuid,
}

pub fn insert_many(data: &[InsertMany], user_id: Uuid) -> InsertStatement {
    let mut query = Query::insert()
        .into_table(UserBookmarkTag::Table)
        .columns([
            UserBookmarkTag::UserBookmarkId,
            UserBookmarkTag::TagId,
            UserBookmarkTag::UserId,
        ])
        .on_conflict(
            OnConflict::columns([UserBookmarkTag::UserBookmarkId, UserBookmarkTag::TagId])
                .do_nothing()
                .to_owned(),
        )
        .to_owned();

    for pbt in data {
        query.values_panic([
            pbt.user_bookmark_id.into(),
            pbt.tag_id.into(),
            user_id.into(),
        ]);
    }

    query
}

pub fn delete_many_not_in_titles(titles: &[String], user_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(UserBookmarkTag::Table)
        .and_where(Expr::col(UserBookmarkTag::UserId).eq(user_id))
        .and_where(
            Expr::col(UserBookmarkTag::TagId)
                .in_subquery(build_titles_subquery(titles, user_id))
                .not(),
        )
        .to_owned()
}
