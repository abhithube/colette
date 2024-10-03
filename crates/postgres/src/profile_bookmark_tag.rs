use sea_query::{Expr, OnConflict, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{types::Uuid, PgExecutor};

use crate::tag::Tag;

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

pub async fn insert_many<'a, E: PgExecutor<'a>>(
    executor: E,
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

pub async fn delete_many_by_titles<'a, E: PgExecutor<'a>>(
    executor: E,
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
        .from_table(ProfileBookmarkTag::Table)
        .and_where(Expr::col(ProfileBookmarkTag::TagId).in_subquery(subquery))
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(())
}

pub async fn delete_many_not_in_ids<'a, E: PgExecutor<'a>>(
    executor: E,
    ids: Vec<Uuid>,
    profile_id: Uuid,
) -> sqlx::Result<()> {
    let query = Query::delete()
        .from_table(ProfileBookmarkTag::Table)
        .and_where(Expr::col(ProfileBookmarkTag::ProfileId).eq(profile_id))
        .and_where(Expr::col(ProfileBookmarkTag::TagId).is_not_in(ids))
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(())
}
