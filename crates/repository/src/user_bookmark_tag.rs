use std::fmt::Write;

use sea_query::{DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Query};
use uuid::Uuid;

use crate::tag::{build_titles_subquery, Tag};

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

pub fn insert_many(user_bookmark_id: Uuid, titles: &[String], user_id: Uuid) -> InsertStatement {
    Query::insert()
        .into_table(UserBookmarkTag::Table)
        .columns([
            UserBookmarkTag::UserBookmarkId,
            UserBookmarkTag::TagId,
            UserBookmarkTag::UserId,
        ])
        .select_from(
            Query::select()
                .expr(Expr::val(user_bookmark_id))
                .column(Tag::Id)
                .column(Tag::UserId)
                .from(Tag::Table)
                .and_where(Expr::col(Tag::UserId).eq(user_id))
                .and_where(Expr::col(Tag::Title).is_in(titles))
                .to_owned(),
        )
        .unwrap()
        .on_conflict(
            OnConflict::columns([UserBookmarkTag::UserBookmarkId, UserBookmarkTag::TagId])
                .do_nothing()
                .to_owned(),
        )
        .to_owned()
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
