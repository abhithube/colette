use chrono::{DateTime, FixedOffset};
use colette_core::{
    bookmarks::{
        BookmarksCreateData, BookmarksFindManyParams, BookmarksRepository, BookmarksUpdateData,
        Error,
    },
    common::FindOneParams,
    Bookmark,
};
use colette_entities::{
    bookmark, profile_bookmark, profile_bookmark_tag, tag, PbWithBookmarkAndTags,
    ProfileBookmarkToTag,
};
use sea_orm::{
    sea_query::OnConflict, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, LoaderTrait,
    ModelTrait, QueryFilter, QueryOrder, Set, TransactionError, TransactionTrait,
};
use uuid::Uuid;

use crate::PostgresRepository;

#[async_trait::async_trait]
impl BookmarksRepository for PostgresRepository {
    async fn find_many_bookmarks(
        &self,
        params: BookmarksFindManyParams,
    ) -> Result<Vec<Bookmark>, Error> {
        let models = profile_bookmark::Entity::find()
            .find_also_related(bookmark::Entity)
            .filter(profile_bookmark::Column::ProfileId.eq(params.profile_id))
            .order_by_asc(bookmark::Column::Title)
            .order_by_asc(profile_bookmark::Column::Id)
            .all(&self.db)
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
                &self.db,
            )
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let bookmarks = models
            .into_iter()
            .zip(tag_models.into_iter())
            .map(|((pb, bookmark), tags)| {
                Bookmark::from(PbWithBookmarkAndTags { pb, bookmark, tags })
            })
            .collect::<Vec<_>>();

        Ok(bookmarks)
    }

    async fn find_one_bookmark(&self, params: FindOneParams) -> Result<Bookmark, Error> {
        find_by_id(&self.db, params).await
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

                    find_by_id(
                        txn,
                        FindOneParams {
                            id: pb_id,
                            profile_id: data.profile_id,
                        },
                    )
                    .await
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
        params: FindOneParams,
        data: BookmarksUpdateData,
    ) -> Result<Bookmark, Error> {
        self.db
            .transaction::<_, Bookmark, Error>(|txn| {
                Box::pin(async move {
                    let Some(pb_model) = profile_bookmark::Entity::find_by_id(params.id)
                        .filter(profile_bookmark::Column::ProfileId.eq(params.profile_id))
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };

                    if let Some(tags) = data.tags {
                        let active_models = tags
                            .clone()
                            .into_iter()
                            .map(|title| tag::ActiveModel {
                                id: Set(Uuid::new_v4()),
                                title: Set(title.clone()),
                                slug: Set(slug::slugify(title)),
                                profile_id: Set(params.profile_id),
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
                                profile_id: Set(params.profile_id),
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

                    find_by_id(txn, params).await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn delete_bookmark(&self, params: FindOneParams) -> Result<(), Error> {
        let result = profile_bookmark::Entity::delete_by_id(params.id)
            .filter(profile_bookmark::Column::ProfileId.eq(params.profile_id))
            .exec(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

async fn find_by_id<Db: ConnectionTrait>(
    db: &Db,
    params: FindOneParams,
) -> Result<Bookmark, Error> {
    let Some((pb_model, Some(bookmark_model))) = profile_bookmark::Entity::find_by_id(params.id)
        .find_also_related(bookmark::Entity)
        .filter(profile_bookmark::Column::ProfileId.eq(params.profile_id))
        .one(db)
        .await
        .map_err(|e| Error::Unknown(e.into()))?
    else {
        return Err(Error::NotFound(params.id));
    };

    let tag_models = pb_model
        .find_linked(ProfileBookmarkToTag)
        .order_by_asc(tag::Column::Title)
        .all(db)
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

    let bookmark = Bookmark::from(PbWithBookmarkAndTags {
        pb: pb_model,
        bookmark: bookmark_model,
        tags: tag_models,
    });

    Ok(bookmark)
}
