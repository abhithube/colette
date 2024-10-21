use std::fmt::Write;

use sea_query::{DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Query};
use uuid::Uuid;

use crate::tag::build_titles_subquery;

pub enum ProfileFeedTag {
    Table,
    ProfileFeedId,
    TagId,
    ProfileId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for ProfileFeedTag {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "profile_feed_tags",
                Self::ProfileFeedId => "profile_feed_id",
                Self::TagId => "tag_id",
                Self::ProfileId => "profile_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

pub struct InsertMany {
    pub profile_feed_id: Uuid,
    pub tag_id: Uuid,
}

pub fn insert_many(data: Vec<InsertMany>, profile_id: Uuid) -> InsertStatement {
    let mut query = Query::insert()
        .into_table(ProfileFeedTag::Table)
        .columns([
            ProfileFeedTag::ProfileFeedId,
            ProfileFeedTag::TagId,
            ProfileFeedTag::ProfileId,
        ])
        .on_conflict(
            OnConflict::columns([ProfileFeedTag::ProfileFeedId, ProfileFeedTag::TagId])
                .do_nothing()
                .to_owned(),
        )
        .to_owned();

    for pft in data {
        query.values_panic([
            pft.profile_feed_id.into(),
            pft.tag_id.into(),
            profile_id.into(),
        ]);
    }

    query
}

pub fn delete_many_in_titles(titles: &[String], profile_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(ProfileFeedTag::Table)
        .and_where(
            Expr::col(ProfileFeedTag::TagId).in_subquery(build_titles_subquery(titles, profile_id)),
        )
        .to_owned()
}

pub fn delete_many_not_in_titles(titles: &[String], profile_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(ProfileFeedTag::Table)
        .and_where(Expr::col(ProfileFeedTag::ProfileId).eq(profile_id))
        .and_where(
            Expr::col(ProfileFeedTag::TagId)
                .in_subquery(build_titles_subquery(titles, profile_id))
                .not(),
        )
        .to_owned()
}
