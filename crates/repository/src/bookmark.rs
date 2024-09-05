use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
use colette_core::{
    bookmark::{
        BookmarkCreateData, BookmarkFindManyFilters, BookmarkRepository, BookmarkUpdateData,
        Cursor, Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    Bookmark,
};
use colette_entity::PbWithBookmarkAndTags;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, DbErr, IntoActiveModel,
    TransactionError, TransactionTrait,
};
use uuid::Uuid;

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
        self.db
            .transaction::<_, Bookmark, Error>(|txn| {
                Box::pin(async move {
                    let result = query::bookmark::insert(
                        txn,
                        data.url,
                        data.bookmark.title,
                        data.bookmark.thumbnail.map(String::from),
                        data.bookmark.published.map(DateTime::<FixedOffset>::from),
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
                        data.collection_id,
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

                    find_by_id(txn, IdParams::new(pb_id, data.profile_id)).await
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
impl Updatable for BookmarkSqlRepository {
    type Params = IdParams;
    type Data = BookmarkUpdateData;
    type Output = Result<Bookmark, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        self.db
            .transaction::<_, Bookmark, Error>(|txn| {
                Box::pin(async move {
                    let Some(pb_model) =
                        query::profile_bookmark::select_by_id(txn, params.id, params.profile_id)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };

                    if let Some(tags) = data.tags {
                        query::tag::insert_many(
                            txn,
                            tags.iter()
                                .map(|e| query::tag::InsertMany {
                                    id: Uuid::new_v4(),
                                    title: e.to_owned(),
                                })
                                .collect(),
                            params.profile_id,
                        )
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                        let tag_models = query::tag::select_by_tags(txn, &tags)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                        let tag_ids = tag_models.iter().map(|e| e.id).collect::<Vec<_>>();

                        query::profile_bookmark_tag::delete_many_not_in(txn, tag_ids.clone())
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        let insert_many = tag_ids
                            .iter()
                            .map(|e| query::profile_bookmark_tag::InsertMany {
                                profile_bookmark_id: pb_model.id,
                                tag_id: *e,
                            })
                            .collect::<Vec<_>>();

                        query::profile_bookmark_tag::insert_many(
                            txn,
                            insert_many,
                            params.profile_id,
                        )
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;
                    }

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

                    if let Some(collection_id) = data.collection_id {
                        active_model.collection_id.set_if_not_equals(collection_id);
                    }

                    if active_model.is_changed() {
                        active_model
                            .update(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    find_by_id(txn, params).await
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

async fn find<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<BookmarkFindManyFilters>,
) -> Result<Vec<Bookmark>, Error> {
    let models = query::profile_bookmark::select_with_bookmark(
        db,
        id,
        profile_id,
        limit.map(|e| e + 1),
        cursor,
        filters,
    )
    .await
    .map(|e| {
        e.into_iter()
            .filter_map(|(pb, bookmark_opt)| bookmark_opt.map(|feed| (pb, feed)))
            .collect::<Vec<_>>()
    })
    .map_err(|e| Error::Unknown(e.into()))?;
    let pb_models = models.iter().map(|e| e.0.to_owned()).collect::<Vec<_>>();

    let tag_models = query::profile_bookmark::load_tags(db, pb_models)
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

    let bookmarks = models
        .into_iter()
        .zip(tag_models.into_iter())
        .map(|((pb, bookmark), tags)| Bookmark::from(PbWithBookmarkAndTags { pb, bookmark, tags }))
        .collect::<Vec<_>>();

    Ok(bookmarks)
}

pub async fn find_by_id<Db: ConnectionTrait>(db: &Db, params: IdParams) -> Result<Bookmark, Error> {
    let bookmarks = find(db, Some(params.id), params.profile_id, None, None, None).await?;

    bookmarks.first().cloned().ok_or(Error::NotFound(params.id))
}
