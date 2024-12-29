use std::fmt::Write;

use sea_query::{DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Query};
use uuid::Uuid;

use crate::tag::build_titles_subquery;

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

pub struct InsertMany {
    pub user_feed_id: Uuid,
    pub tag_id: Uuid,
}

pub fn insert_many(data: &[InsertMany], user_id: Uuid) -> InsertStatement {
    let mut query = Query::insert()
        .into_table(UserFeedTag::Table)
        .columns([
            UserFeedTag::UserFeedId,
            UserFeedTag::TagId,
            UserFeedTag::UserId,
        ])
        .on_conflict(
            OnConflict::columns([UserFeedTag::UserFeedId, UserFeedTag::TagId])
                .do_nothing()
                .to_owned(),
        )
        .to_owned();

    for pft in data {
        query.values_panic([pft.user_feed_id.into(), pft.tag_id.into(), user_id.into()]);
    }

    query
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
