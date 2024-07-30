use chrono::{DateTime, FixedOffset};
use colette_core::{
    bookmarks::{
        BookmarksCreateData, BookmarksFindManyParams, BookmarksRepository, BookmarksUpdateData,
        Error,
    },
    common::{self, FindOneParams, UpdateTagList},
    Bookmark,
};
use colette_entities::{bookmark, bookmark_tag, BookmarkToTag, BookmarkWithTags};
use sea_orm::{
    sea_query::OnConflict, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter,
    QueryOrder, QuerySelect, QueryTrait, Set, TransactionError, TransactionTrait,
};
use uuid::Uuid;

pub struct BookmarksSqlRepository {
    db: DatabaseConnection,
}

impl BookmarksSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl BookmarksRepository for BookmarksSqlRepository {
    async fn find_many(&self, params: BookmarksFindManyParams) -> Result<Vec<Bookmark>, Error> {
        let query = bookmark::Entity::find()
            .filter(bookmark::Column::ProfileId.eq(params.profile_id))
            .apply_if(params.published_at, |query, v| {
                query.filter(bookmark::Column::PublishedAt.lt(v))
            })
            .order_by_desc(bookmark::Column::PublishedAt)
            .order_by_asc(bookmark::Column::Title)
            .order_by_asc(bookmark::Column::Id)
            .limit(params.limit as u64);

        match params.with_tags {
            true => query
                .find_with_linked(BookmarkToTag)
                .all(&self.db)
                .await
                .map(|e| {
                    e.into_iter()
                        .map(|(bookmark, tags)| BookmarkWithTags(bookmark, Some(tags)).into())
                        .collect()
                }),
            false => query.all(&self.db).await.map(|e| {
                e.into_iter()
                    .map(|bookmark| BookmarkWithTags(bookmark, None).into())
                    .collect()
            }),
        }
        .map_err(|e| Error::Unknown(e.into()))
    }

    async fn create(&self, data: BookmarksCreateData) -> Result<Bookmark, Error> {
        let model = bookmark::ActiveModel {
            id: Set(Uuid::new_v4()),
            link: Set(data.url.clone()),
            title: Set(data.bookmark.title),
            thumbnail_url: Set(data.bookmark.thumbnail.map(String::from)),
            published_at: Set(data.bookmark.published.map(DateTime::<FixedOffset>::from)),
            author: Set(data.bookmark.author),
            profile_id: Set(data.profile_id),
            ..Default::default()
        };

        let bookmark = bookmark::Entity::insert(model)
            .on_conflict(
                OnConflict::columns([bookmark::Column::ProfileId, bookmark::Column::Link])
                    .update_columns([
                        bookmark::Column::Title,
                        bookmark::Column::ThumbnailUrl,
                        bookmark::Column::PublishedAt,
                        bookmark::Column::Author,
                    ])
                    .to_owned(),
            )
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(BookmarkWithTags(bookmark, None).into())
    }

    async fn update(
        &self,
        params: FindOneParams,
        data: BookmarksUpdateData,
    ) -> Result<Bookmark, Error> {
        self.db
            .transaction::<_, Bookmark, Error>(|txn| {
                Box::pin(async move {
                    let Some(bookmark) = bookmark::Entity::find_by_id(params.id)
                        .filter(bookmark::Column::ProfileId.eq(params.profile_id))
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };

                    if let Some(tags) = data.tags {
                        match tags {
                            UpdateTagList::Add(tag_ids) => {
                                let models = tag_ids
                                    .into_iter()
                                    .map(|id| bookmark_tag::ActiveModel {
                                        tag_id: Set(id),
                                        bookmark_id: Set(params.id),
                                    })
                                    .collect::<Vec<_>>();

                                bookmark_tag::Entity::insert_many(models)
                                    .on_conflict(
                                        OnConflict::columns([
                                            bookmark_tag::Column::BookmarkId,
                                            bookmark_tag::Column::TagId,
                                        ])
                                        .do_nothing()
                                        .to_owned(),
                                    )
                                    .exec_without_returning(txn)
                                    .await
                                    .map_err(|e| Error::Unknown(e.into()))?;
                            }
                            UpdateTagList::Remove(tag_ids) => {
                                bookmark_tag::Entity::delete_many()
                                    .filter(bookmark_tag::Column::BookmarkId.eq(params.id))
                                    .filter(bookmark_tag::Column::TagId.is_in(tag_ids))
                                    .exec(txn)
                                    .await
                                    .map_err(|e| Error::Unknown(e.into()))?;
                            }
                            UpdateTagList::Set(tag_ids) => {
                                bookmark_tag::Entity::delete_many()
                                    .filter(bookmark_tag::Column::BookmarkId.eq(params.id))
                                    .exec(txn)
                                    .await
                                    .map_err(|e| Error::Unknown(e.into()))?;

                                let models = tag_ids
                                    .into_iter()
                                    .map(|id| bookmark_tag::ActiveModel {
                                        tag_id: Set(id),
                                        bookmark_id: Set(params.id),
                                    })
                                    .collect::<Vec<_>>();

                                bookmark_tag::Entity::insert_many(models)
                                    .exec_without_returning(txn)
                                    .await
                                    .map_err(|e| Error::Unknown(e.into()))?;
                            }
                        }
                    }

                    let tags = bookmark
                        .find_linked(BookmarkToTag)
                        .all(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    Ok(BookmarkWithTags(bookmark, Some(tags)).into())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn delete(&self, params: common::FindOneParams) -> Result<(), Error> {
        let result = bookmark::Entity::delete_by_id(params.id)
            .filter(bookmark::Column::ProfileId.eq(params.profile_id))
            .exec(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}
