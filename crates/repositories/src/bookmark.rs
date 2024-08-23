use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
use colette_core::{
    bookmark::{
        BookmarkCreateData, BookmarkFindManyFilters, BookmarkRepository, BookmarkUpdateData, Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, Paginated, Updatable},
    Bookmark,
};
use colette_entities::PbWithBookmarkAndTags;
use colette_utils::base_64;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, DbErr, IntoActiveModel,
    TransactionError, TransactionTrait,
};
use uuid::Uuid;

use crate::queries;

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
                    let result = queries::bookmark::insert(
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

                    let prev = queries::profile_bookmark::select_last(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let pb_id = match queries::profile_bookmark::insert(
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
                        Err(DbErr::RecordNotFound(_)) => {
                            let Some(model) = queries::profile_bookmark::select_by_unique_index(
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
                        queries::profile_bookmark::select_by_id(txn, params.id, params.profile_id)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };

                    if let Some(tags) = data.tags {
                        queries::tag::insert_many(
                            txn,
                            tags.iter()
                                .map(|e| queries::tag::InsertMany {
                                    id: Uuid::new_v4(),
                                    title: e.to_owned(),
                                })
                                .collect(),
                            params.profile_id,
                        )
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                        let tag_models = queries::tag::select_by_tags(txn, &tags)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                        let tag_ids = tag_models.iter().map(|e| e.id).collect::<Vec<_>>();

                        queries::profile_bookmark_tag::delete_many_not_in(txn, tag_ids.clone())
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        let insert_many = tag_ids
                            .iter()
                            .map(|e| queries::profile_bookmark_tag::InsertMany {
                                profile_bookmark_id: pb_model.id,
                                tag_id: *e,
                            })
                            .collect::<Vec<_>>();

                        queries::profile_bookmark_tag::insert_many(
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
                        queries::profile_bookmark::update_many_sort_indexes(
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
                        queries::profile_bookmark::select_by_id(txn, params.id, params.profile_id)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };

                    queries::profile_bookmark::decrement_many_sort_indexes(
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
        cursor: Option<String>,
        filters: Option<BookmarkFindManyFilters>,
    ) -> Result<Paginated<Bookmark>, Error> {
        find(&self.db, None, profile_id, limit, cursor, filters).await
    }
}

async fn find<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor_raw: Option<String>,
    filters: Option<BookmarkFindManyFilters>,
) -> Result<Paginated<Bookmark>, Error> {
    let models = queries::profile_bookmark::select_with_bookmark(
        db,
        id,
        profile_id,
        limit.map(|e| e + 1),
        cursor_raw.and_then(|e| base_64::decode::<Cursor>(&e).ok()),
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

    let tag_models = queries::profile_bookmark::load_tags(db, pb_models)
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

    let mut bookmarks = models
        .into_iter()
        .zip(tag_models.into_iter())
        .map(|((pb, bookmark), tags)| Bookmark::from(PbWithBookmarkAndTags { pb, bookmark, tags }))
        .collect::<Vec<_>>();
    let mut cursor: Option<String> = None;

    if let Some(limit) = limit {
        let limit = limit as usize;
        if bookmarks.len() > limit {
            bookmarks = bookmarks.into_iter().take(limit).collect();

            if let Some(last) = bookmarks.last() {
                let c = Cursor {
                    sort_index: last.sort_index,
                };
                let encoded = base_64::encode(&c)?;

                cursor = Some(encoded);
            }
        }
    }

    Ok(Paginated::<Bookmark> {
        cursor,
        data: bookmarks,
    })
}

pub async fn find_by_id<Db: ConnectionTrait>(db: &Db, params: IdParams) -> Result<Bookmark, Error> {
    let bookmarks = find(db, Some(params.id), params.profile_id, None, None, None).await?;

    bookmarks
        .data
        .first()
        .cloned()
        .ok_or(Error::NotFound(params.id))
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub sort_index: u32,
}