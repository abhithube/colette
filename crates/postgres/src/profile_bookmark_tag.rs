use sea_query::{Expr, OnConflict, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{types::Uuid, PgExecutor};

use crate::tag::build_titles_subquery;

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub enum ProfileBookmarkTag {
    Table,
    ProfileBookmarkId,
    TagId,
    ProfileId,
    CreatedAt,
    UpdatedAt,
}

pub struct InsertMany {
    pub profile_bookmark_id: Uuid,
    pub tag_id: Uuid,
}

pub async fn insert_many(
    executor: impl PgExecutor<'_>,
    data: Vec<InsertMany>,
    profile_id: Uuid,
) -> sqlx::Result<()> {
    let mut query = Query::insert()
        .into_table(ProfileBookmarkTag::Table)
        .columns([
            ProfileBookmarkTag::ProfileBookmarkId,
            ProfileBookmarkTag::TagId,
            ProfileBookmarkTag::ProfileId,
        ])
        .on_conflict(
            OnConflict::columns([
                ProfileBookmarkTag::ProfileBookmarkId,
                ProfileBookmarkTag::TagId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .to_owned();

    for pft in data {
        query.values_panic([
            pft.profile_bookmark_id.into(),
            pft.tag_id.into(),
            profile_id.into(),
        ]);
    }

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(())
}

pub async fn delete_many_in_titles(
    executor: impl PgExecutor<'_>,
    titles: &[String],
    profile_id: Uuid,
) -> sqlx::Result<()> {
    let query = Query::delete()
        .from_table(ProfileBookmarkTag::Table)
        .and_where(
            Expr::col(ProfileBookmarkTag::TagId)
                .in_subquery(build_titles_subquery(titles, profile_id)),
        )
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(())
}

pub async fn delete_many_not_in_titles(
    executor: impl PgExecutor<'_>,
    titles: &[String],
    profile_id: Uuid,
) -> sqlx::Result<()> {
    let query = Query::delete()
        .from_table(ProfileBookmarkTag::Table)
        .and_where(Expr::col(ProfileBookmarkTag::ProfileId).eq(profile_id))
        .and_where(
            Expr::col(ProfileBookmarkTag::TagId)
                .in_subquery(build_titles_subquery(titles, profile_id))
                .not(),
        )
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(())
}
