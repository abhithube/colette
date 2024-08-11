use chrono::{DateTime, FixedOffset};
use colette_core::{
    bookmarks::{
        BookmarksCreateData, BookmarksFindManyFilters, BookmarksRepository, BookmarksUpdateData,
        Error,
    },
    common::{Paginated, PAGINATION_LIMIT},
    Bookmark,
};
use colette_entities::{
    bookmark, profile_bookmark, profile_bookmark_tag, tag, PbWithBookmarkAndTags,
};
use sea_orm::{
    prelude::Expr, sea_query::OnConflict, ColumnTrait, Condition, ConnectionTrait, DbErr,
    EntityTrait, LoaderTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionError,
    TransactionTrait,
};
use uuid::Uuid;

use crate::{utils, SqlRepository};

#[async_trait::async_trait]
impl BookmarksRepository for SqlRepository {
    async fn find_many_bookmarks(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<String>,
        filters: Option<BookmarksFindManyFilters>,
    ) -> Result<Paginated<Bookmark>, Error> {
        find(&self.db, None, profile_id, limit, cursor, filters).await
    }

    async fn find_one_bookmark(&self, id: Uuid, profile_id: Uuid) -> Result<Bookmark, Error> {
        find_by_id(&self.db, id, profile_id).await
    }

    async fn create_bookmark(&self, data: BookmarksCreateData) -> Result<Bookmark, Error> {
        self.db
            .transaction::<_, Bookmark, Error>(|txn| {
                Box::pin(async move {
                    let active_model = bookmark::ActiveModel {
                        link: Set(data.url),
                        title: Set(data.bookmark.title),
                        thumbnail_url: Set(data.bookmark.thumbnail.map(String::from)),
                        published_at: Set(data
                            .bookmark
                            .published
                            .map(DateTime::<FixedOffset>::from)),
                        author: Set(data.bookmark.author),
                        ..Default::default()
                    };

                    let result = bookmark::Entity::insert(active_model)
                        .on_conflict(
                            OnConflict::column(bookmark::Column::Link)
                                .update_columns([
                                    bookmark::Column::Title,
                                    bookmark::Column::ThumbnailUrl,
                                    bookmark::Column::PublishedAt,
                                    bookmark::Column::Author,
                                ])
                                .to_owned(),
                        )
                        .exec(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;
                    let bookmark_id = result.last_insert_id;

                    let active_model = profile_bookmark::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        profile_id: Set(data.profile_id),
                        bookmark_id: Set(bookmark_id),
                        ..Default::default()
                    };

                    let pb_id = match profile_bookmark::Entity::insert(active_model)
                        .on_conflict(
                            OnConflict::columns([
                                profile_bookmark::Column::ProfileId,
                                profile_bookmark::Column::BookmarkId,
                            ])
                            .do_nothing()
                            .to_owned(),
                        )
                        .exec(txn)
                        .await
                    {
                        Ok(result) => Ok(result.last_insert_id),
                        Err(DbErr::RecordNotFound(_)) => {
                            let Some(model) = profile_bookmark::Entity::find()
                                .filter(profile_bookmark::Column::ProfileId.eq(data.profile_id))
                                .filter(profile_bookmark::Column::BookmarkId.eq(bookmark_id))
                                .one(txn)
                                .await
                                .map_err(|e| Error::Unknown(e.into()))?
                            else {
                                return Err(Error::Unknown(anyhow::anyhow!(
                                    "Failed to fetch created profile bookmark"
                                )));
                            };

                            Ok(model.id)
                        }
                        Err(e) => Err(Error::Unknown(e.into())),
                    }?;

                    find_by_id(txn, pb_id, data.profile_id).await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn update_bookmark(
        &self,
        id: Uuid,
        profile_id: Uuid,
        data: BookmarksUpdateData,
    ) -> Result<Bookmark, Error> {
        self.db
            .transaction::<_, Bookmark, Error>(|txn| {
                Box::pin(async move {
                    let Some(pb_model) = profile_bookmark::Entity::find_by_id(id)
                        .filter(profile_bookmark::Column::ProfileId.eq(profile_id))
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(id));
                    };

                    if let Some(tags) = data.tags {
                        let active_models = tags
                            .clone()
                            .into_iter()
                            .map(|title| tag::ActiveModel {
                                id: Set(Uuid::new_v4()),
                                title: Set(title.clone()),
                                profile_id: Set(profile_id),
                                ..Default::default()
                            })
                            .collect::<Vec<_>>();

                        tag::Entity::insert_many(active_models)
                            .on_empty_do_nothing()
                            .on_conflict(
                                OnConflict::columns([tag::Column::ProfileId, tag::Column::Title])
                                    .do_nothing()
                                    .to_owned(),
                            )
                            .exec(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        let tag_models = tag::Entity::find()
                            .filter(tag::Column::Title.is_in(&tags))
                            .all(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                        let tag_ids = tag_models.iter().map(|e| e.id).collect::<Vec<_>>();

                        profile_bookmark_tag::Entity::delete_many()
                            .filter(profile_bookmark_tag::Column::TagId.is_not_in(tag_ids.clone()))
                            .exec(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        let active_models = tag_ids
                            .into_iter()
                            .map(|tag_id| profile_bookmark_tag::ActiveModel {
                                profile_bookmark_id: Set(pb_model.id),
                                tag_id: Set(tag_id),
                                profile_id: Set(profile_id),
                                ..Default::default()
                            })
                            .collect::<Vec<_>>();

                        profile_bookmark_tag::Entity::insert_many(active_models)
                            .on_empty_do_nothing()
                            .on_conflict(
                                OnConflict::columns([
                                    profile_bookmark_tag::Column::ProfileBookmarkId,
                                    profile_bookmark_tag::Column::TagId,
                                ])
                                .do_nothing()
                                .to_owned(),
                            )
                            .exec(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    find_by_id(txn, id, profile_id).await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn delete_bookmark(&self, id: Uuid, profile_id: Uuid) -> Result<(), Error> {
        let result = profile_bookmark::Entity::delete_by_id(id)
            .filter(profile_bookmark::Column::ProfileId.eq(profile_id))
            .exec(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(id));
        }

        Ok(())
    }
}

async fn find<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor_raw: Option<String>,
    _filters: Option<BookmarksFindManyFilters>,
) -> Result<Paginated<Bookmark>, Error> {
    let mut conditions = Condition::all().add(profile_bookmark::Column::ProfileId.eq(profile_id));
    if let Some(id) = id {
        conditions = conditions.add(profile_bookmark::Column::Id.eq(id));
    }
    if let Some(raw) = cursor_raw.as_deref() {
        let cursor = utils::decode_cursor::<Cursor>(raw).map_err(|e| Error::Unknown(e.into()))?;

        conditions = conditions.add(
            Expr::tuple([
                Expr::col((bookmark::Entity, bookmark::Column::Title)).into(),
                Expr::col((profile_bookmark::Entity, profile_bookmark::Column::Id)).into(),
            ])
            .gt(Expr::tuple([
                Expr::value(cursor.title),
                Expr::value(cursor.id),
            ])),
        );
    }

    let models = profile_bookmark::Entity::find()
        .find_also_related(bookmark::Entity)
        .filter(conditions)
        .order_by_asc(bookmark::Column::Title)
        .order_by_asc(profile_bookmark::Column::Id)
        .limit(limit)
        .all(db)
        .await
        .map(|e| {
            e.into_iter()
                .filter_map(|(pb, bookmark_opt)| bookmark_opt.map(|feed| (pb, feed)))
                .collect::<Vec<_>>()
        })
        .map_err(|e| Error::Unknown(e.into()))?;
    let pb_models = models.clone().into_iter().map(|e| e.0).collect::<Vec<_>>();

    let tag_models = pb_models
        .load_many_to_many(
            tag::Entity::find().order_by_asc(tag::Column::Title),
            profile_bookmark_tag::Entity,
            db,
        )
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

    let mut bookmarks = models
        .into_iter()
        .zip(tag_models.into_iter())
        .map(|((pb, bookmark), tags)| Bookmark::from(PbWithBookmarkAndTags { pb, bookmark, tags }))
        .collect::<Vec<_>>();
    let mut cursor: Option<String> = None;

    if bookmarks.len() > PAGINATION_LIMIT {
        bookmarks = bookmarks.into_iter().take(PAGINATION_LIMIT).collect();

        if let Some(last) = bookmarks.last() {
            let c = Cursor {
                id: last.id,
                title: last.title.to_owned(),
            };
            let encoded = utils::encode_cursor(&c).map_err(|e| Error::Unknown(e.into()))?;

            cursor = Some(encoded);
        }
    }

    Ok(Paginated::<Bookmark> {
        cursor,
        data: bookmarks,
    })
}

pub async fn find_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
) -> Result<Bookmark, Error> {
    let bookmarks = find(db, Some(id), profile_id, None, None, None).await?;

    bookmarks.data.first().cloned().ok_or(Error::NotFound(id))
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
struct Cursor {
    pub id: Uuid,
    pub title: String,
}