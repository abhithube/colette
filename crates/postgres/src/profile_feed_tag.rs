use sea_query::{Expr, OnConflict, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{types::Uuid, PgExecutor};

use crate::tag::Tag;

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub enum ProfileFeedTag {
    Table,
    ProfileFeedId,
    TagId,
    ProfileId,
    CreatedAt,
    UpdatedAt,
}

pub struct InsertMany {
    pub profile_feed_id: Uuid,
    pub tag_id: Uuid,
}

pub async fn insert_many(
    executor: impl PgExecutor<'_>,
    data: Vec<InsertMany>,
    profile_id: Uuid,
) -> sqlx::Result<()> {
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

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(())
}

pub async fn delete_many_by_titles(
    executor: impl PgExecutor<'_>,
    titles: &[String],
    profile_id: Uuid,
) -> sqlx::Result<()> {
    let subquery = Query::select()
        .column(Tag::Id)
        .from(Tag::Table)
        .and_where(Expr::col(Tag::ProfileId).eq(profile_id))
        .and_where(Expr::col(Tag::Title).is_in(titles))
        .to_owned();

    let query = Query::delete()
        .from_table(ProfileFeedTag::Table)
        .and_where(Expr::col(ProfileFeedTag::TagId).in_subquery(subquery))
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(())
}

pub async fn delete_many_not_in_ids(
    executor: impl PgExecutor<'_>,
    ids: Vec<Uuid>,
    profile_id: Uuid,
) -> sqlx::Result<()> {
    let query = Query::delete()
        .from_table(ProfileFeedTag::Table)
        .and_where(Expr::col(ProfileFeedTag::ProfileId).eq(profile_id))
        .and_where(Expr::col(ProfileFeedTag::TagId).is_not_in(ids))
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(())
}
