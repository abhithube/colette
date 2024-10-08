use colette_core::feed::Cursor;
use colette_sql::profile_feed;
use sea_query::{
    extension::postgres::PgExpr, Alias, CommonTableExpression, Expr, Func, JoinType, PgFunc,
    PostgresQueryBuilder, Query, WithClause,
};
use sea_query_binder::SqlxBinder;
use sqlx::{
    types::{Json, Uuid},
    PgExecutor,
};

use crate::{
    common::{JsonbArrayElements, JsonbBuildObject},
    feed::Feed,
    profile_feed_entry::ProfileFeedEntry,
    profile_feed_tag::ProfileFeedTag,
    tag::Tag,
};

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub(crate) enum ProfileFeed {
    Table,
    Id,
    Title,
    Pinned,
    ProfileId,
    FeedId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct FeedSelect {
    pub id: Uuid,
    pub link: String,
    pub title: Option<String>,
    pub pinned: bool,
    pub original_title: String,
    pub url: Option<String>,
    pub tags: Option<Json<Vec<TagSelect>>>,
    pub unread_count: i64,
}

impl From<FeedSelect> for colette_core::Feed {
    fn from(value: FeedSelect) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            pinned: value.pinned,
            original_title: value.original_title,
            url: value.url,
            tags: value
                .tags
                .map(|e| e.0.into_iter().map(|e| e.into()).collect()),
            unread_count: Some(value.unread_count),
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
            bookmark_count: None,
            feed_count: None,
        }
    }
}

pub async fn find(
    executor: impl PgExecutor<'_>,
    id: Option<Uuid>,
    profile_id: Uuid,
    pinned: Option<bool>,
    tags: Option<Vec<String>>,
    cursor: Option<Cursor>,
    limit: Option<u64>,
) -> sqlx::Result<Vec<colette_core::Feed>> {
    let unread_count = Alias::new("unread_count");
    let pf_id = Alias::new("pf_id");

    let unread_count_cte = Query::select()
        .expr_as(
            Expr::col((ProfileFeed::Table, ProfileFeed::Id)),
            pf_id.clone(),
        )
        .expr_as(
            Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::Id)).count(),
            unread_count.clone(),
        )
        .from(ProfileFeed::Table)
        .join(
            JoinType::InnerJoin,
            ProfileFeedEntry::Table,
            Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::ProfileFeedId))
                .eq(Expr::col((ProfileFeed::Table, ProfileFeed::Id))),
        )
        .group_by_col((ProfileFeed::Table, ProfileFeed::Id))
        .to_owned();

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
            Expr::col((ProfileFeed::Table, ProfileFeed::Id)),
            pf_id.clone(),
        )
        .expr_as(jsonb_agg, a_tags.clone())
        .from(ProfileFeed::Table)
        .join(
            JoinType::InnerJoin,
            ProfileFeedTag::Table,
            Expr::col((ProfileFeedTag::Table, ProfileFeedTag::ProfileFeedId))
                .eq(Expr::col((ProfileFeed::Table, ProfileFeed::Id))),
        )
        .join(
            JoinType::InnerJoin,
            Tag::Table,
            Expr::col((Tag::Table, Tag::Id))
                .eq(Expr::col((ProfileFeedTag::Table, ProfileFeedTag::TagId))),
        )
        .group_by_col((ProfileFeed::Table, ProfileFeed::Id))
        .to_owned();

    let json_tags = Alias::new("json_tags");
    let unread_counts = Alias::new("unread_counts");

    let mut select = Query::select()
        .columns([
            (ProfileFeed::Table, ProfileFeed::Id),
            (ProfileFeed::Table, ProfileFeed::Title),
            (ProfileFeed::Table, ProfileFeed::Pinned),
        ])
        .columns([(Feed::Table, Feed::Link), (Feed::Table, Feed::Url)])
        .expr_as(
            Expr::col((Feed::Table, Feed::Title)),
            Alias::new("original_title"),
        )
        .expr_as(
            Func::coalesce([Expr::col((json_tags.clone(), a_tags.clone())).into()]),
            a_tags.clone(),
        )
        .expr_as(
            Func::coalesce([
                Expr::col((unread_counts.clone(), unread_count.clone())).into(),
                Expr::val(0).into(),
            ]),
            unread_count,
        )
        .from(ProfileFeed::Table)
        .join(
            JoinType::Join,
            Feed::Table,
            Expr::col((Feed::Table, Feed::Id))
                .eq(Expr::col((ProfileFeed::Table, ProfileFeed::FeedId))),
        )
        .join(
            JoinType::LeftJoin,
            json_tags.clone(),
            Expr::col((json_tags.clone(), pf_id.clone()))
                .eq(Expr::col((ProfileFeed::Table, ProfileFeed::Id))),
        )
        .join(
            JoinType::LeftJoin,
            unread_counts.clone(),
            Expr::col((unread_counts.clone(), pf_id.clone()))
                .eq(Expr::col((ProfileFeed::Table, ProfileFeed::Id))),
        )
        .and_where(Expr::col((ProfileFeed::Table, ProfileFeed::ProfileId)).eq(profile_id))
        .and_where_option(id.map(|e| Expr::col((ProfileFeed::Table, ProfileFeed::Id)).eq(e)))
        .and_where_option(
            pinned.map(|e| Expr::col((ProfileFeed::Table, ProfileFeed::Pinned)).eq(e)),
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
            Expr::tuple([
                Func::coalesce([
                    Expr::col((ProfileFeed::Table, ProfileFeed::Title)).into(),
                    Expr::col((Feed::Table, Feed::Title)).into(),
                ])
                .into(),
                Expr::col((ProfileFeed::Table, ProfileFeed::Id)).into(),
            ])
            .lt(Expr::tuple([
                Expr::val(e.title).into(),
                Expr::val(e.id).into(),
            ]))
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
            .cte(
                CommonTableExpression::new()
                    .query(unread_count_cte)
                    .table_name(unread_counts)
                    .to_owned(),
            )
            .to_owned(),
    );

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, FeedSelect, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(|e| e.into()).collect())
}

pub async fn select_by_unique_index(
    executor: impl PgExecutor<'_>,
    profile_id: Uuid,
    feed_id: i32,
) -> sqlx::Result<Uuid> {
    let query = profile_feed::select_by_unique_index(profile_id, feed_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
        .fetch_one(executor)
        .await
}

pub async fn insert(
    executor: impl PgExecutor<'_>,
    id: Uuid,
    pinned: Option<bool>,
    feed_id: i32,
    profile_id: Uuid,
) -> sqlx::Result<Uuid> {
    let query = profile_feed::insert(id, pinned, feed_id, profile_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
        .fetch_one(executor)
        .await
}

pub async fn update(
    executor: impl PgExecutor<'_>,
    id: Uuid,
    profile_id: Uuid,
    title: Option<Option<String>>,
    pinned: Option<bool>,
) -> sqlx::Result<()> {
    let query = profile_feed::update(id, profile_id, title, pinned);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

pub async fn delete(executor: impl PgExecutor<'_>, id: Uuid, profile_id: Uuid) -> sqlx::Result<()> {
    let query = profile_feed::delete(id, profile_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}
