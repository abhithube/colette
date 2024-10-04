use anyhow::anyhow;
use colette_core::{
    bookmark::{
        BookmarkCreateData, BookmarkFindManyFilters, BookmarkRepository, BookmarkUpdateData,
        Cursor, Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, TagsLinkAction, TagsLinkData, Updatable},
    Bookmark,
};
use sea_orm::{
    prelude::{DateTimeWithTimeZone, Uuid},
    sqlx, ActiveModelTrait, DatabaseConnection, DbErr, IntoActiveModel, TransactionError,
    TransactionTrait,
};

use crate::query;

pub struct BookmarkSqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl BookmarkSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for BookmarkSqlRepository {
    type Params = IdParams;
    type Output = Result<Bookmark, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        find_by_id(&self.db, params).await
    }
}

#[async_trait::async_trait]
impl Creatable for BookmarkSqlRepository {
    type Data = BookmarkCreateData;
    type Output = Result<Bookmark, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let id = self
            .db
            .transaction::<_, Uuid, Error>(|txn| {
                Box::pin(async move {
                    let result = query::bookmark::insert(
                        txn,
                        data.url,
                        data.bookmark.title,
                        data.bookmark.thumbnail.map(String::from),
                        data.bookmark.published.map(DateTimeWithTimeZone::from),
                        data.bookmark.author,
                    )
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;
                    let bookmark_id = result.last_insert_id;

                    let prev = query::profile_bookmark::select_last(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let pb_id = match query::profile_bookmark::insert(
                        txn,
                        Uuid::new_v4(),
                        prev.map(|e| e.sort_index + 1).unwrap_or_default(),
                        data.profile_id,
                        bookmark_id,
                    )
                    .await
                    {
                        Ok(result) => Ok(result.last_insert_id),
                        Err(DbErr::RecordNotInserted) => {
                            let Some(model) = query::profile_bookmark::select_by_unique_index(
                                txn,
                                data.profile_id,
                                bookmark_id,
                            )
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                            else {
                                return Err(Error::Unknown(anyhow!(
                                    "Failed to fetch created profile bookmark"
                                )));
                            };

                            Ok(model.id)
                        }
                        Err(e) => Err(Error::Unknown(e.into())),
                    }?;

                    Ok(pb_id)
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })?;

        if let Some(tags) = data.tags {
            link_tags(
                self.db.get_postgres_connection_pool(),
                id,
                tags,
                data.profile_id,
            )
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        }

        find_by_id(&self.db, IdParams::new(id, data.profile_id)).await
    }
}

#[async_trait::async_trait]
impl Updatable for BookmarkSqlRepository {
    type Params = IdParams;
    type Data = BookmarkUpdateData;
    type Output = Result<Bookmark, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let id = self
            .db
            .transaction::<_, Uuid, Error>(|txn| {
                Box::pin(async move {
                    let Some(pb_model) =
                        query::profile_bookmark::select_by_id(txn, params.id, params.profile_id)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };

                    let pb_id = pb_model.id;
                    let old_sort_index = pb_model.sort_index;
                    let mut active_model = pb_model.into_active_model();

                    if let Some(sort_index) = data.sort_index {
                        query::profile_bookmark::update_many_sort_indexes(
                            txn,
                            sort_index as i32,
                            old_sort_index,
                            params.profile_id,
                        )
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                        active_model.sort_index.set_if_not_equals(sort_index as i32);
                    }

                    if active_model.is_changed() {
                        active_model
                            .update(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    Ok(pb_id)
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })?;

        if let Some(tags) = data.tags {
            link_tags(
                self.db.get_postgres_connection_pool(),
                id,
                tags,
                params.profile_id,
            )
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        }

        find_by_id(&self.db, params).await
    }
}

#[async_trait::async_trait]
impl Deletable for BookmarkSqlRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        self.db
            .transaction::<_, (), Error>(|txn| {
                Box::pin(async move {
                    let Some(pb_model) =
                        query::profile_bookmark::select_by_id(txn, params.id, params.profile_id)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };

                    query::profile_bookmark::decrement_many_sort_indexes(
                        txn,
                        pb_model.sort_index,
                        params.profile_id,
                    )
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

                    pb_model
                        .into_active_model()
                        .delete(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }
}

#[async_trait::async_trait]
impl BookmarkRepository for BookmarkSqlRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
        filters: Option<BookmarkFindManyFilters>,
    ) -> Result<Vec<Bookmark>, Error> {
        find(&self.db, None, profile_id, limit, cursor, filters).await
    }
}

pub(crate) async fn find(
    db: &DatabaseConnection,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<BookmarkFindManyFilters>,
) -> Result<Vec<Bookmark>, Error> {
    let mut tags: Option<Vec<String>> = None;

    if let Some(filters) = filters {
        tags = filters.tags;
    }

    colette_postgres::profile_bookmark::find(
        db.get_postgres_connection_pool(),
        id,
        profile_id,
        tags,
        cursor,
        limit,
    )
    .await
    .map_err(|e| Error::Unknown(e.into()))
}

pub async fn find_by_id(db: &DatabaseConnection, params: IdParams) -> Result<Bookmark, Error> {
    let mut bookmarks = find(db, Some(params.id), params.profile_id, None, None, None).await?;
    if bookmarks.is_empty() {
        return Err(Error::NotFound(params.id));
    }

    Ok(bookmarks.swap_remove(0))
}

pub(crate) async fn link_tags(
    pool: &sqlx::PgPool,
    profile_bookmark_id: Uuid,
    tags: TagsLinkData,
    profile_id: Uuid,
) -> sqlx::Result<()> {
    if let TagsLinkAction::Remove = tags.action {
        return colette_postgres::profile_bookmark_tag::delete_many_in_titles(
            pool, &tags.data, profile_id,
        )
        .await;
    }

    if let TagsLinkAction::Set = tags.action {
        colette_postgres::profile_bookmark_tag::delete_many_not_in_titles(
            pool, &tags.data, profile_id,
        )
        .await?;
    }

    colette_postgres::tag::insert_many(
        pool,
        tags.data
            .iter()
            .map(|e| colette_postgres::tag::InsertMany {
                id: Uuid::new_v4(),
                title: e.to_owned(),
            })
            .collect(),
        profile_id,
    )
    .await?;

    let tag_ids = colette_postgres::tag::select_ids_by_titles(pool, &tags.data, profile_id).await?;

    let insert_many = tag_ids
        .into_iter()
        .map(|e| colette_postgres::profile_bookmark_tag::InsertMany {
            profile_bookmark_id,
            tag_id: e,
        })
        .collect::<Vec<_>>();

    colette_postgres::profile_bookmark_tag::insert_many(pool, insert_many, profile_id).await
}
