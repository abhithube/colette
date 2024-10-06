use colette_core::bookmark::Cursor;
use sea_query::{
    extension::postgres::PgExpr, Alias, CommonTableExpression, Expr, Func, JoinType, OnConflict,
    PgFunc, PostgresQueryBuilder, Query, WithClause,
};
use sea_query_binder::SqlxBinder;
use sqlx::{
    types::{
        chrono::{DateTime, Utc},
        Json, Uuid,
    },
    FromRow, PgExecutor, Row,
};

use crate::{
    bookmark::Bookmark,
    common::{JsonbArrayElements, JsonbBuildObject},
    profile_bookmark_tag::ProfileBookmarkTag,
    tag::Tag,
};

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub(crate) enum ProfileBookmark {
    Table,
    Id,
    SortIndex,
    ProfileId,
    BookmarkId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, Clone)]
struct BookmarkSelect {
    pub id: Uuid,
    pub link: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub sort_index: i32,
    pub tags: Option<Json<Vec<TagSelect>>>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for BookmarkSelect {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        let bookmark = Self {
            id: row.try_get("id")?,
            link: row.try_get("link")?,
            title: row.try_get("title")?,
            thumbnail_url: row.try_get("thumbnail_url")?,
            published_at: row.try_get("published_at")?,
            author: row.try_get("author")?,
            sort_index: row.try_get("sort_index")?,
            tags: row.try_get("tags")?,
        };

        Ok(bookmark)
    }
}

impl From<BookmarkSelect> for colette_core::Bookmark {
    fn from(value: BookmarkSelect) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author,
            sort_index: value.sort_index as u32,
            tags: value
                .tags
                .map(|e| e.0.into_iter().map(|e| e.into()).collect()),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TagSelect {
    id: Uuid,
    title: String,
}

impl From<TagSelect> for colette_core::Tag {
    fn from(value: TagSelect) -> Self {
        Self {
            id: value.id,
            title: value.title,
            parent_id: None,
            depth: 0,
            direct: None,
            bookmark_count: None,
            feed_count: None,
        }
    }
}

pub async fn find(
    executor: impl PgExecutor<'_>,
    id: Option<Uuid>,
    profile_id: Uuid,
    tags: Option<Vec<String>>,
    cursor: Option<Cursor>,
    limit: Option<u64>,
) -> sqlx::Result<Vec<colette_core::Bookmark>> {
    let pf_id = Alias::new("pf_id");

    let jsonb_agg = Expr::cust_with_exprs(
        "JSONB_AGG($1 ORDER BY $2) FILTER (WHERE $3)",
        [
            Func::cust(JsonbBuildObject)
                .args([
                    Expr::val("id").into(),
                    Expr::col((Tag::Table, Tag::Id)).into(),
                    Expr::val("title").into(),
                    Expr::col((Tag::Table, Tag::Title)).into(),
                ])
                .into(),
            Expr::col((Tag::Table, Tag::Title)).into(),
            Expr::col((Tag::Table, Tag::Id)).is_not_null(),
        ],
    )
    .to_owned();

    let a_tags = Alias::new("tags");

    let json_tags_cte = Query::select()
        .expr_as(
            Expr::col((ProfileBookmark::Table, ProfileBookmark::Id)),
            pf_id.clone(),
        )
        .expr_as(jsonb_agg, a_tags.clone())
        .from(ProfileBookmark::Table)
        .join(
            JoinType::InnerJoin,
            ProfileBookmarkTag::Table,
            Expr::col((
                ProfileBookmarkTag::Table,
                ProfileBookmarkTag::ProfileBookmarkId,
            ))
            .eq(Expr::col((ProfileBookmark::Table, ProfileBookmark::Id))),
        )
        .join(
            JoinType::InnerJoin,
            Tag::Table,
            Expr::col((Tag::Table, Tag::Id)).eq(Expr::col((
                ProfileBookmarkTag::Table,
                ProfileBookmarkTag::TagId,
            ))),
        )
        .group_by_col((ProfileBookmark::Table, ProfileBookmark::Id))
        .to_owned();

    let json_tags = Alias::new("json_tags");

    let mut select = Query::select()
        .columns([
            (ProfileBookmark::Table, ProfileBookmark::Id),
            (ProfileBookmark::Table, ProfileBookmark::SortIndex),
        ])
        .columns([
            (Bookmark::Table, Bookmark::Link),
            (Bookmark::Table, Bookmark::Title),
            (Bookmark::Table, Bookmark::ThumbnailUrl),
            (Bookmark::Table, Bookmark::PublishedAt),
            (Bookmark::Table, Bookmark::Author),
        ])
        .expr_as(
            Func::coalesce([Expr::col((json_tags.clone(), a_tags.clone())).into()]),
            a_tags.clone(),
        )
        .from(ProfileBookmark::Table)
        .join(
            JoinType::Join,
            Bookmark::Table,
            Expr::col((Bookmark::Table, Bookmark::Id)).eq(Expr::col((
                ProfileBookmark::Table,
                ProfileBookmark::BookmarkId,
            ))),
        )
        .join(
            JoinType::LeftJoin,
            json_tags.clone(),
            Expr::col((json_tags.clone(), pf_id.clone()))
                .eq(Expr::col((ProfileBookmark::Table, ProfileBookmark::Id))),
        )
        .and_where(Expr::col((ProfileBookmark::Table, ProfileBookmark::ProfileId)).eq(profile_id))
        .and_where_option(
            id.map(|e| Expr::col((ProfileBookmark::Table, ProfileBookmark::Id)).eq(e)),
        )
        .and_where_option(tags.map(|e| {
            let t = Alias::new("t");

            Expr::exists(
                Query::select()
                    .expr(Expr::val(1))
                    .from_function(
                        Func::cust(JsonbArrayElements)
                            .arg(Expr::col((json_tags.clone(), a_tags.clone()))),
                        t.clone(),
                    )
                    .and_where(Expr::col(t).get_json_field("title").eq(PgFunc::any(e)))
                    .to_owned(),
            )
        }))
        .and_where_option(cursor.map(|e| {
            Expr::col((ProfileBookmark::Table, ProfileBookmark::SortIndex))
                .gt(Expr::val(e.sort_index))
        }))
        .to_owned();

    if let Some(limit) = limit {
        select.limit(limit);
    }

    let query = select.with(
        WithClause::new()
            .cte(
                CommonTableExpression::new()
                    .query(json_tags_cte)
                    .table_name(json_tags)
                    .to_owned(),
            )
            .to_owned(),
    );

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, BookmarkSelect, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(|e| e.into()).collect())
}

pub async fn select_by_unique_index(
    executor: impl PgExecutor<'_>,
    profile_id: Uuid,
    bookmark_id: i32,
) -> sqlx::Result<Uuid> {
    let query = Query::select()
        .column(ProfileBookmark::Id)
        .from(ProfileBookmark::Table)
        .and_where(Expr::col(ProfileBookmark::ProfileId).eq(profile_id))
        .and_where(Expr::col(ProfileBookmark::BookmarkId).eq(bookmark_id))
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let row = sqlx::query_with(&sql, values).fetch_one(executor).await?;

    row.try_get("id")
}

pub async fn insert(
    executor: impl PgExecutor<'_>,
    id: Uuid,
    bookmark_id: i32,
    profile_id: Uuid,
) -> sqlx::Result<Uuid> {
    let select = Query::select()
        .expr(Expr::val(id))
        .expr(
            Expr::expr(Func::coalesce([
                Expr::col(ProfileBookmark::SortIndex).max(),
                Expr::val(0).into(),
            ]))
            .add(1),
        )
        .expr(Expr::val(bookmark_id))
        .expr(Expr::val(profile_id))
        .from(ProfileBookmark::Table)
        .to_owned();

    let query = Query::insert()
        .into_table(ProfileBookmark::Table)
        .columns([
            ProfileBookmark::Id,
            ProfileBookmark::SortIndex,
            ProfileBookmark::BookmarkId,
            ProfileBookmark::ProfileId,
        ])
        .select_from(select)
        .unwrap()
        .on_conflict(
            OnConflict::columns([ProfileBookmark::ProfileId, ProfileBookmark::BookmarkId])
                .do_nothing()
                .to_owned(),
        )
        .returning_col(ProfileBookmark::Id)
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let row = sqlx::query_with(&sql, values).fetch_one(executor).await?;

    row.try_get("id")
}

pub async fn decrement_many_sort_indexes(
    executor: impl PgExecutor<'_>,
    profile_id: Uuid,
    sort_index: u32,
) -> sqlx::Result<()> {
    let query = Query::update()
        .table(ProfileBookmark::Table)
        .value(
            ProfileBookmark::SortIndex,
            Expr::col(ProfileBookmark::SortIndex).sub(1),
        )
        .and_where(Expr::col(ProfileBookmark::ProfileId).eq(profile_id))
        .and_where(Expr::col(ProfileBookmark::SortIndex).gt(sort_index))
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(())
}

pub async fn delete(executor: impl PgExecutor<'_>, id: Uuid, profile_id: Uuid) -> sqlx::Result<()> {
    let query = Query::delete()
        .from_table(ProfileBookmark::Table)
        .and_where(Expr::col((ProfileBookmark::Table, ProfileBookmark::Id)).eq(id))
        .and_where(Expr::col((ProfileBookmark::Table, ProfileBookmark::ProfileId)).eq(profile_id))
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}
