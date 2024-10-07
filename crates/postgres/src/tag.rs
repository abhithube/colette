use colette_core::tag::{Cursor, TagFindManyFilters, TagType};
use sea_query::{Alias, Expr, OnConflict, Order, PostgresQueryBuilder, Query, SelectStatement};
use sea_query_binder::SqlxBinder;
use sqlx::{types::Uuid, PgExecutor, Row};

use crate::{profile_bookmark_tag::ProfileBookmarkTag, profile_feed_tag::ProfileFeedTag};

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub(crate) enum Tag {
    Table,
    Id,
    Title,
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

pub async fn select_by_title(
    executor: impl PgExecutor<'_>,
    title: String,
    profile_id: Uuid,
) -> sqlx::Result<Uuid> {
    let query = Query::select()
        .column(Tag::Id)
        .from(Tag::Table)
        .and_where(Expr::col(Tag::ProfileId).eq(profile_id))
        .and_where(Expr::col(Tag::Title).eq(title))
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let row = sqlx::query_with(&sql, values).fetch_one(executor).await?;

    row.try_get("id")
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

pub async fn insert(
    executor: impl PgExecutor<'_>,
    id: Uuid,
    title: String,
    profile_id: Uuid,
) -> sqlx::Result<Uuid> {
    let query = Query::insert()
        .into_table(Tag::Table)
        .columns([Tag::Id, Tag::Title, Tag::ProfileId])
        .values_panic([id.into(), title.into(), profile_id.into()])
        .returning_col(Tag::Id)
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let row = sqlx::query_with(&sql, values).fetch_one(executor).await?;

    row.try_get("id")
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

pub async fn update(
    executor: impl PgExecutor<'_>,
    id: Uuid,
    profile_id: Uuid,
    title: Option<String>,
) -> sqlx::Result<()> {
    let mut query = Query::update()
        .table(Tag::Table)
        .and_where(Expr::col(Tag::Id).eq(id))
        .and_where(Expr::col(Tag::ProfileId).eq(profile_id))
        .to_owned();

    if let Some(title) = title {
        query.value(Tag::Title, title);
    }

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

pub async fn delete_by_id(
    executor: impl PgExecutor<'_>,
    id: Uuid,
    profile_id: Uuid,
) -> sqlx::Result<()> {
    let query = Query::delete()
        .from_table(Tag::Table)
        .and_where(Expr::col((Tag::Table, Tag::Id)).eq(id))
        .and_where(Expr::col((Tag::Table, Tag::ProfileId)).eq(profile_id))
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

pub async fn delete_many(executor: impl PgExecutor<'_>) -> sqlx::Result<u64> {
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

    let query = Query::delete()
        .from_table(Tag::Table)
        .and_where(Expr::exists(feed_subquery).not())
        .and_where(Expr::exists(bookmark_subquery).not())
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(result.rows_affected())
}

pub(crate) fn build_titles_subquery(titles: &[String], profile_id: Uuid) -> SelectStatement {
    Query::select()
        .column(Tag::Id)
        .from(Tag::Table)
        .and_where(Expr::col(Tag::ProfileId).eq(profile_id))
        .and_where(Expr::col(Tag::Title).is_in(titles))
        .to_owned()
}
