use chrono::{DateTime, FixedOffset, Utc};
use colette_core::{
    bookmarks::{
        BookmarksCreateData, BookmarksFindManyParams, BookmarksRepository, BookmarksUpdateData,
        Error,
    },
    common::FindOneParams,
};
use colette_entities::{bookmark, profile_bookmark, profile_bookmark_tag, tag};
use sea_orm::{
    sea_query::OnConflict, ColumnTrait, DbErr, EntityTrait, QueryFilter, Set, TransactionError,
    TransactionTrait,
};
use sqlx::types::Json;
use uuid::Uuid;

use crate::{tags::Tag, PostgresRepository};

#[async_trait::async_trait]
impl BookmarksRepository for PostgresRepository {
    async fn find_many_bookmarks(
        &self,
        params: BookmarksFindManyParams,
    ) -> Result<Vec<colette_core::Bookmark>, Error> {
        sqlx::query_file_as!(
            Bookmark,
            "queries/bookmarks/find_many.sql",
            params.profile_id,
            params.limit,
            params.tags.as_deref()
        )
        .fetch_all(self.db.get_postgres_connection_pool())
        .await
        .map(|e| e.into_iter().map(colette_core::Bookmark::from).collect())
        .map_err(|e| Error::Unknown(e.into()))
    }

    async fn find_one_bookmark(
        &self,
        params: FindOneParams,
    ) -> Result<colette_core::Bookmark, Error> {
        sqlx::query_file_as!(
            Bookmark,
            "queries/bookmarks/find_one.sql",
            params.id,
            params.profile_id
        )
        .fetch_one(self.db.get_postgres_connection_pool())
        .await
        .map(colette_core::Bookmark::from)
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }

    async fn create_bookmark(
        &self,
        data: BookmarksCreateData,
    ) -> Result<colette_core::Bookmark, Error> {
        let id = self
            .db
            .transaction::<_, Uuid, Error>(|txn| {
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

                    Ok(pb_id)
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })?;

        self.find_one_bookmark(FindOneParams {
            id,
            profile_id: data.profile_id,
        })
        .await
    }

    async fn update_bookmark(
        &self,
        params: FindOneParams,
        data: BookmarksUpdateData,
    ) -> Result<colette_core::Bookmark, Error> {
        self.db
            .transaction::<_, (), Error>(|txn| {
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
                                title: Set(title),
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

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })?;

        self.find_one_bookmark(params).await
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

#[derive(Clone, Debug)]
struct Bookmark {
    id: Uuid,
    link: String,
    title: String,
    thumbnail_url: Option<String>,
    published_at: Option<DateTime<Utc>>,
    author: Option<String>,
    tags: Json<Vec<Tag>>,
}

impl From<Bookmark> for colette_core::Bookmark {
    fn from(value: Bookmark) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author,
            tags: value
                .tags
                .0
                .into_iter()
                .map(colette_core::Tag::from)
                .collect(),
        }
    }
}
