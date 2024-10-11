use colette_core::tag::{Cursor, TagFindManyFilters, TagType};
use sea_query::{
    Alias, DeleteStatement, Expr, InsertStatement, OnConflict, Order, Query, SelectStatement,
    UpdateStatement,
};
use uuid::Uuid;

use crate::{profile_bookmark_tag::ProfileBookmarkTag, profile_feed_tag::ProfileFeedTag};

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub enum Tag {
    Table,
    Id,
    Title,
    ProfileId,
    CreatedAt,
    UpdatedAt,
}

pub struct InsertMany {
    pub id: Uuid,
    pub title: String,
}

pub fn select(
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<TagFindManyFilters>,
) -> SelectStatement {
    let mut query = Query::select()
        .column((Tag::Table, Tag::Id))
        .column((Tag::Table, Tag::Title))
        .expr_as(
            Expr::col(ProfileFeedTag::ProfileFeedId).count(),
            Alias::new("feed_count"),
        )
        .expr_as(
            Expr::col(ProfileBookmarkTag::ProfileBookmarkId).count(),
            Alias::new("bookmark_count"),
        )
        .from(Tag::Table)
        .left_join(
            ProfileFeedTag::Table,
            Expr::col((ProfileFeedTag::Table, ProfileFeedTag::TagId))
                .eq(Expr::col((Tag::Table, Tag::Id))),
        )
        .left_join(
            ProfileBookmarkTag::Table,
            Expr::col((ProfileBookmarkTag::Table, ProfileBookmarkTag::TagId))
                .eq(Expr::col((Tag::Table, Tag::Id))),
        )
        .and_where(Expr::col((Tag::Table, Tag::ProfileId)).eq(profile_id))
        .and_where_option(id.map(|e| Expr::col((Tag::Table, Tag::Id)).eq(e)))
        .and_where_option(cursor.map(|e| Expr::col(Tag::Title).gt(e.title)))
        .group_by_columns([(Tag::Table, Tag::Id), (Tag::Table, Tag::Title)])
        .order_by((Tag::Table, Tag::Title), Order::Asc)
        .to_owned();

    if let Some(filters) = filters {
        match filters.tag_type {
            TagType::Bookmarks => {
                query.and_having(
                    Expr::expr(Expr::col(ProfileBookmarkTag::ProfileBookmarkId).count()).gt(0),
                );
            }
            TagType::Feeds => {
                query
                    .and_having(Expr::expr(Expr::col(ProfileFeedTag::ProfileFeedId).count()).gt(0));
            }
            _ => {}
        };

        query.and_where_option(
            filters
                .feed_id
                .map(|e| Expr::col(ProfileFeedTag::ProfileFeedId).eq(e)),
        );
        query.and_where_option(
            filters
                .bookmark_id
                .map(|e| Expr::col(ProfileBookmarkTag::ProfileBookmarkId).eq(e)),
        );
    }
    if let Some(limit) = limit {
        query.limit(limit);
    }

    query
}

pub fn select_by_title(title: String, profile_id: Uuid) -> SelectStatement {
    Query::select()
        .column(Tag::Id)
        .from(Tag::Table)
        .and_where(Expr::col(Tag::ProfileId).eq(profile_id))
        .and_where(Expr::col(Tag::Title).eq(title))
        .to_owned()
}

pub fn select_ids_by_titles(titles: &[String], profile_id: Uuid) -> SelectStatement {
    Query::select()
        .column(Tag::Id)
        .from(Tag::Table)
        .and_where(Expr::col(Tag::ProfileId).eq(profile_id))
        .and_where(Expr::col(Tag::Title).is_in(titles))
        .to_owned()
}

pub fn insert(id: Uuid, title: String, profile_id: Uuid) -> InsertStatement {
    Query::insert()
        .into_table(Tag::Table)
        .columns([Tag::Id, Tag::Title, Tag::ProfileId])
        .values_panic([id.into(), title.into(), profile_id.into()])
        .to_owned()
}

pub fn insert_many(data: Vec<InsertMany>, profile_id: Uuid) -> InsertStatement {
    let mut query = Query::insert()
        .into_table(Tag::Table)
        .columns([Tag::Id, Tag::Title, Tag::ProfileId])
        .on_conflict(
            OnConflict::columns([Tag::ProfileId, Tag::Title])
                .do_nothing()
                .to_owned(),
        )
        .to_owned();

    for t in data {
        query.values_panic([t.id.into(), t.title.into(), profile_id.into()]);
    }

    query
}

pub fn update(id: Uuid, profile_id: Uuid, title: Option<String>) -> UpdateStatement {
    let mut query = Query::update()
        .table(Tag::Table)
        .and_where(Expr::col(Tag::Id).eq(id))
        .and_where(Expr::col(Tag::ProfileId).eq(profile_id))
        .to_owned();

    if let Some(title) = title {
        query.value(Tag::Title, title);
    }

    query
}

pub fn delete_by_id(id: Uuid, profile_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(Tag::Table)
        .and_where(Expr::col((Tag::Table, Tag::Id)).eq(id))
        .and_where(Expr::col((Tag::Table, Tag::ProfileId)).eq(profile_id))
        .to_owned()
}

pub fn delete_many() -> DeleteStatement {
    let feed_subquery = Query::select()
        .from(ProfileFeedTag::Table)
        .and_where(
            Expr::col((ProfileFeedTag::Table, ProfileFeedTag::TagId)).equals((Tag::Table, Tag::Id)),
        )
        .to_owned();

    let bookmark_subquery = Query::select()
        .from(ProfileBookmarkTag::Table)
        .and_where(
            Expr::col((ProfileBookmarkTag::Table, ProfileBookmarkTag::TagId))
                .equals((Tag::Table, Tag::Id)),
        )
        .to_owned();

    Query::delete()
        .from_table(Tag::Table)
        .and_where(Expr::exists(feed_subquery).not())
        .and_where(Expr::exists(bookmark_subquery).not())
        .to_owned()
}

pub(crate) fn build_titles_subquery(titles: &[String], profile_id: Uuid) -> SelectStatement {
    Query::select()
        .column(Tag::Id)
        .from(Tag::Table)
        .and_where(Expr::col(Tag::ProfileId).eq(profile_id))
        .and_where(Expr::col(Tag::Title).is_in(titles))
        .to_owned()
}
