use colette_core::tag::{Cursor, TagFindManyFilters, TagType};
use sea_query::{
    Alias, CommonTableExpression, Expr, OnConflict, Order, PostgresQueryBuilder, Query, UnionType,
};
use sea_query_binder::SqlxBinder;
use sqlx::{types::Uuid, PgExecutor};

use crate::{profile_bookmark_tag::ProfileBookmarkTag, profile_feed_tag::ProfileFeedTag};

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub(crate) enum Tag {
    Table,
    Id,
    Title,
    ParentId,
    ProfileId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct TagSelect {
    id: Uuid,
    title: String,
    bookmark_count: i64,
    feed_count: i64,
}

impl From<TagSelect> for colette_core::Tag {
    fn from(value: TagSelect) -> Self {
        Self {
            id: value.id,
            title: value.title,
            parent_id: None,
            depth: 0,
            direct: None,
            bookmark_count: Some(value.bookmark_count),
            feed_count: Some(value.feed_count),
        }
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TagSelectId {
    pub id: Uuid,
}

pub struct InsertMany {
    pub id: Uuid,
    pub title: String,
}

pub async fn select(
    executor: impl PgExecutor<'_>,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<TagFindManyFilters>,
) -> sqlx::Result<Vec<colette_core::Tag>> {
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

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, TagSelect, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(|e| e.into()).collect())
}

pub async fn select_ids_by_titles(
    executor: impl PgExecutor<'_>,
    titles: &[String],
    profile_id: Uuid,
) -> sqlx::Result<Vec<Uuid>> {
    let query = Query::select()
        .column(Tag::Id)
        .from(Tag::Table)
        .and_where(Expr::col(Tag::ProfileId).eq(profile_id))
        .and_where(Expr::col(Tag::Title).is_in(titles))
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, TagSelectId, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(|e| e.id).collect())
}

pub async fn select_ids_by_pf_id(
    executor: impl PgExecutor<'_>,
    profile_feed_id: Uuid,
    profile_id: Uuid,
) -> sqlx::Result<Vec<Uuid>> {
    let query = Query::select()
        .column(Tag::Id)
        .from(Tag::Table)
        .inner_join(
            ProfileFeedTag::Table,
            Expr::col((ProfileFeedTag::Table, ProfileFeedTag::TagId))
                .eq(Expr::col((Tag::Table, Tag::Id)))
                .and(
                    Expr::col((ProfileFeedTag::Table, ProfileFeedTag::ProfileFeedId))
                        .eq(profile_feed_id),
                ),
        )
        .and_where(Expr::col(Tag::ProfileId).eq(profile_id))
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, TagSelectId, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(|e| e.id).collect())
}

pub async fn insert_many(
    executor: impl PgExecutor<'_>,
    data: Vec<InsertMany>,
    profile_id: Uuid,
) -> sqlx::Result<()> {
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

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(())
}

pub async fn prune_tag_list(
    executor: impl PgExecutor<'_>,
    tag_ids: Vec<Uuid>,
    profile_id: Uuid,
) -> sqlx::Result<Vec<Uuid>> {
    let tag_hierarchy = Alias::new("tag_hierarchy");

    let subquery = Query::select()
        .expr(Expr::val(1))
        .from(tag_hierarchy.clone())
        .and_where(
            Expr::col((tag_hierarchy.clone(), Tag::ParentId)).eq(Expr::col((Tag::Table, Tag::Id))),
        )
        .and_where(Expr::col((tag_hierarchy, Tag::Id)).is_in(tag_ids.clone()))
        .to_owned();

    let final_query = Query::select()
        .distinct()
        .column(Tag::Id)
        .from(Tag::Table)
        .and_where(
            Expr::col(Tag::Id)
                .is_in(tag_ids)
                .and(Expr::exists(subquery).not()),
        )
        .to_owned();

    let query = final_query.with(
        Query::with()
            .cte(build_tag_recursive_cte(profile_id))
            .recursive(true)
            .to_owned(),
    );

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, TagSelectId, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(|e| e.id).collect())
}

pub(crate) fn build_tag_recursive_cte(profile_id: Uuid) -> CommonTableExpression {
    let tag_hierarchy = Alias::new("tag_hierarchy");
    let depth = Alias::new("depth");

    let mut base_query = Query::select()
        .columns([Tag::Id, Tag::Title, Tag::ParentId])
        .expr_as(Expr::val(1), depth.clone())
        .from(Tag::Table)
        .and_where(Expr::col(Tag::ProfileId).eq(profile_id))
        .and_where(Expr::col(Tag::ParentId).is_null())
        .to_owned();

    let recursive_query = Query::select()
        .columns([
            (Tag::Table, Tag::Id),
            (Tag::Table, Tag::Title),
            (Tag::Table, Tag::ParentId),
        ])
        .expr(Expr::col((tag_hierarchy.clone(), depth)).add(1))
        .from(Tag::Table)
        .inner_join(
            tag_hierarchy.clone(),
            Expr::col((tag_hierarchy.clone(), Tag::Id)).eq(Expr::col((Tag::Table, Tag::ParentId))),
        )
        .to_owned();

    CommonTableExpression::new()
        .query(base_query.union(UnionType::All, recursive_query).to_owned())
        .table_name(tag_hierarchy)
        .to_owned()
}
