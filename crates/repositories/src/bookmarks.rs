use anyhow::anyhow;
use chrono::{DateTime, FixedOffset, Utc};
use colette_core::{
    bookmarks::{
        BookmarksCreateData, BookmarksFindManyParams, BookmarksRepository, BookmarksUpdateData,
        Error,
    },
    common::{self, FindOneParams, UpdateTagList},
    Bookmark,
};
use colette_entities::{bookmark, bookmark_tag};
use sea_orm::{
    sea_query::OnConflict, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, SelectModel, Selector, Set, TransactionError, TransactionTrait,
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
        let mut query = bookmark::Entity::find()
            .select_only()
            .columns(BOOKMARK_COLUMNS)
            .filter(bookmark::Column::ProfileId.eq(params.profile_id));

        if let Some(published_at) = params.published_at {
            query = query.filter(bookmark::Column::PublishedAt.lt(published_at))
        }

        query
            .order_by_desc(bookmark::Column::PublishedAt)
            .order_by_asc(bookmark::Column::Title)
            .order_by_asc(bookmark::Column::Id)
            .limit(params.limit as u64)
            .into_model::<BookmarkSelect>()
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Bookmark::from).collect())
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn create(&self, data: BookmarksCreateData) -> Result<Bookmark, Error> {
        self.db
            .transaction::<_, Bookmark, Error>(|txn| {
                Box::pin(async move {
                    let bookmark_model = bookmark::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        link: Set(data.url.clone()),
                        title: Set(data.bookmark.title),
                        thumbnail_url: Set(data.bookmark.thumbnail.map(String::from)),
                        published_at: Set(data
                            .bookmark
                            .published
                            .map(DateTime::<FixedOffset>::from)),
                        author: Set(data.bookmark.author),
                        profile_id: Set(data.profile_id),
                        ..Default::default()
                    };

                    bookmark::Entity::insert(bookmark_model)
                        .on_conflict(
                            OnConflict::columns([
                                bookmark::Column::ProfileId,
                                bookmark::Column::Link,
                            ])
                            .update_columns([
                                bookmark::Column::Title,
                                bookmark::Column::ThumbnailUrl,
                                bookmark::Column::PublishedAt,
                                bookmark::Column::Author,
                            ])
                            .to_owned(),
                        )
                        .exec_without_returning(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let Some(bookmark) = bookmark::Entity::find()
                        .select_only()
                        .columns(BOOKMARK_COLUMNS)
                        .filter(bookmark::Column::ProfileId.eq(data.profile_id))
                        .filter(bookmark::Column::Link.eq(data.url))
                        .into_model::<BookmarkSelect>()
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::Unknown(anyhow!("Failed to fetch created bookmark")));
                    };

                    Ok(bookmark.into())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn update(
        &self,
        params: FindOneParams,
        data: BookmarksUpdateData,
    ) -> Result<Bookmark, Error> {
        self.db
            .transaction::<_, Bookmark, Error>(|txn| {
                Box::pin(async move {
                    let Some(bookmark) = bookmark_by_id(params.id, params.profile_id)
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

                    Ok(bookmark.into())
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

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
struct BookmarkSelect {
    id: Uuid,
    link: String,
    title: String,
    thumbnail_url: Option<String>,
    published_at: Option<DateTime<FixedOffset>>,
    author: Option<String>,
    profile_id: Uuid,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
}

impl From<BookmarkSelect> for Bookmark {
    fn from(value: BookmarkSelect) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            published_at: value.published_at.map(DateTime::<Utc>::from),
            author: value.author,
            thumbnail_url: value.thumbnail_url,
            profile_id: value.profile_id,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
        }
    }
}

const BOOKMARK_COLUMNS: [bookmark::Column; 9] = [
    bookmark::Column::Id,
    bookmark::Column::Link,
    bookmark::Column::Title,
    bookmark::Column::ThumbnailUrl,
    bookmark::Column::PublishedAt,
    bookmark::Column::Author,
    bookmark::Column::ProfileId,
    bookmark::Column::CreatedAt,
    bookmark::Column::UpdatedAt,
];

fn bookmark_by_id(id: Uuid, profile_id: Uuid) -> Selector<SelectModel<BookmarkSelect>> {
    bookmark::Entity::find_by_id(id)
        .select_only()
        .columns(BOOKMARK_COLUMNS)
        .filter(bookmark::Column::ProfileId.eq(profile_id))
        .into_model::<BookmarkSelect>()
}
