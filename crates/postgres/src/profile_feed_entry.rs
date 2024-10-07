use colette_core::feed_entry::Cursor;
use sea_query::{Expr, JoinType, OnConflict, Order, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{
    types::{
        chrono::{DateTime, Utc},
        Uuid,
    },
    PgExecutor,
};

use crate::{
    feed_entry::FeedEntry,
    profile_feed::ProfileFeed,
    profile_feed_tag::ProfileFeedTag,
    smart_feed_filter::{build_case_statement, SmartFeedFilter},
    tag::Tag,
};

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub(crate) enum ProfileFeedEntry {
    Table,
    Id,
    HasRead,
    ProfileFeedId,
    FeedEntryId,
    ProfileId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct EntrySelect {
    id: Uuid,
    link: String,
    title: String,
    published_at: DateTime<Utc>,
    description: Option<String>,
    author: Option<String>,
    thumbnail_url: Option<String>,
    has_read: bool,
    profile_feed_id: Uuid,
}

impl From<EntrySelect> for colette_core::FeedEntry {
    fn from(value: EntrySelect) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            published_at: value.published_at,
            description: value.description,
            author: value.author,
            thumbnail_url: value.thumbnail_url,
            has_read: value.has_read,
            feed_id: value.profile_feed_id,
        }
    }
}

pub struct InsertMany {
    pub id: Uuid,
    pub feed_entry_id: i32,
}

#[allow(clippy::too_many_arguments)]
pub async fn select(
    executor: impl PgExecutor<'_>,
    id: Option<Uuid>,
    profile_id: Uuid,
    feed_id: Option<Uuid>,
    has_read: Option<bool>,
    tags: Option<&[String]>,
    smart_feed_id: Option<Uuid>,
    cursor: Option<Cursor>,
    limit: Option<u64>,
) -> sqlx::Result<Vec<colette_core::FeedEntry>> {
    let mut query = Query::select()
        .columns([
            (ProfileFeedEntry::Table, ProfileFeedEntry::Id),
            (ProfileFeedEntry::Table, ProfileFeedEntry::HasRead),
            (ProfileFeedEntry::Table, ProfileFeedEntry::ProfileFeedId),
        ])
        .columns([
            (FeedEntry::Table, FeedEntry::Link),
            (FeedEntry::Table, FeedEntry::Title),
            (FeedEntry::Table, FeedEntry::PublishedAt),
            (FeedEntry::Table, FeedEntry::Description),
            (FeedEntry::Table, FeedEntry::Author),
            (FeedEntry::Table, FeedEntry::ThumbnailUrl),
        ])
        .from(ProfileFeedEntry::Table)
        .join(
            JoinType::Join,
            FeedEntry::Table,
            Expr::col((FeedEntry::Table, FeedEntry::Id)).eq(Expr::col((
                ProfileFeedEntry::Table,
                ProfileFeedEntry::FeedEntryId,
            ))),
        )
        .and_where(Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::ProfileId)).eq(profile_id))
        .and_where_option(
            id.map(|e| Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::Id)).eq(e)),
        )
        .and_where_option(
            feed_id.map(|e| {
                Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::ProfileFeedId)).eq(e)
            }),
        )
        .and_where_option(
            has_read.map(|e| Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::HasRead)).eq(e)),
        )
        .and_where_option(cursor.map(|e| {
            Expr::tuple([
                Expr::col((FeedEntry::Table, FeedEntry::PublishedAt)).into(),
                Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::Id)).into(),
            ])
            .lt(Expr::tuple([
                Expr::val(e.published_at).into(),
                Expr::val(e.id).into(),
            ]))
        }))
        .order_by((FeedEntry::Table, FeedEntry::PublishedAt), Order::Desc)
        .order_by((ProfileFeedEntry::Table, ProfileFeedEntry::Id), Order::Desc)
        .to_owned();

    if let Some(tags) = tags {
        query
            .join(
                JoinType::InnerJoin,
                ProfileFeedTag::Table,
                Expr::col((ProfileFeedTag::Table, ProfileFeedTag::ProfileFeedId)).eq(Expr::col((
                    ProfileFeedEntry::Table,
                    ProfileFeedEntry::ProfileFeedId,
                ))),
            )
            .join(
                JoinType::InnerJoin,
                ProfileFeed::Table,
                Expr::col((Tag::Table, Tag::Id))
                    .eq(Expr::col((ProfileFeedTag::Table, ProfileFeedTag::TagId))),
            )
            .and_where(Expr::col((Tag::Table, Tag::Title)).is_in(tags));
    }

    if let Some(smart_feed_id) = smart_feed_id {
        query.join(
            JoinType::InnerJoin,
            SmartFeedFilter::Table,
            Expr::col((SmartFeedFilter::Table, SmartFeedFilter::SmartFeedId))
                .eq(Expr::val(smart_feed_id))
                .and(build_case_statement().into()),
        );
    }

    if let Some(limit) = limit {
        query.limit(limit);
    }

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, EntrySelect, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(|e| e.into()).collect())
}

pub async fn insert_many(
    executor: impl PgExecutor<'_>,
    data: Vec<InsertMany>,
    pf_id: Uuid,
    profile_id: Uuid,
) -> sqlx::Result<()> {
    let mut query = Query::insert()
        .into_table(ProfileFeedEntry::Table)
        .columns([
            ProfileFeedEntry::Id,
            ProfileFeedEntry::FeedEntryId,
            ProfileFeedEntry::ProfileFeedId,
            ProfileFeedEntry::ProfileId,
        ])
        .on_conflict(
            OnConflict::columns([
                ProfileFeedEntry::ProfileFeedId,
                ProfileFeedEntry::FeedEntryId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .returning_col(FeedEntry::Id)
        .to_owned();

    for pfe in data {
        query.values_panic([
            pfe.id.into(),
            pfe.feed_entry_id.into(),
            pf_id.into(),
            profile_id.into(),
        ]);
    }

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(())
}

pub async fn update(
    executor: impl PgExecutor<'_>,
    id: Uuid,
    profile_id: Uuid,
    has_read: Option<bool>,
) -> sqlx::Result<()> {
    let mut query = Query::update()
        .table(ProfileFeedEntry::Table)
        .and_where(Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::Id)).eq(id))
        .and_where(Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::ProfileId)).eq(profile_id))
        .to_owned();

    if let Some(has_read) = has_read {
        query.value(ProfileFeedEntry::HasRead, has_read);
    }

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}
