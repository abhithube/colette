use std::fmt::Write;

use sea_query::{DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Query};
use uuid::Uuid;

use crate::tag::{build_titles_subquery, Tag};

#[allow(dead_code)]
pub enum UserFeedTag {
    Table,
    UserFeedId,
    TagId,
    UserId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for UserFeedTag {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "user_feed_tags",
                Self::UserFeedId => "user_feed_id",
                Self::TagId => "tag_id",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

pub fn insert_many(user_feed_id: Uuid, titles: &[String], user_id: Uuid) -> InsertStatement {
    Query::insert()
        .into_table(UserFeedTag::Table)
        .columns([
            UserFeedTag::UserFeedId,
            UserFeedTag::TagId,
            UserFeedTag::UserId,
        ])
        .select_from(
            Query::select()
                .expr(Expr::val(user_feed_id))
                .column(Tag::Id)
                .column(Tag::UserId)
                .from(Tag::Table)
                .and_where(Expr::col(Tag::UserId).eq(user_id))
                .and_where(Expr::col(Tag::Title).is_in(titles))
                .to_owned(),
        )
        .unwrap()
        .on_conflict(
            OnConflict::columns([UserFeedTag::UserFeedId, UserFeedTag::TagId])
                .do_nothing()
                .to_owned(),
        )
        .to_owned()
}

pub fn delete_many_not_in_titles(titles: &[String], user_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(UserFeedTag::Table)
        .and_where(Expr::col(UserFeedTag::UserId).eq(user_id))
        .and_where(
            Expr::col(UserFeedTag::TagId)
                .in_subquery(build_titles_subquery(titles, user_id))
                .not(),
        )
        .to_owned()
}
