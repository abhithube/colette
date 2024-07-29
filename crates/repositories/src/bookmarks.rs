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
use colette_entities::{bookmark, bookmark_tag, collection};
use sea_orm::{
    prelude::Expr, sea_query::OnConflict, ColumnTrait, DatabaseConnection, EntityTrait, JoinType,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, SelectModel, Selector, Set,
    TransactionError, TransactionTrait,
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
            .expr_as(
                Expr::case(
                    collection::Column::IsDefault.eq(true),
                    Expr::value(Option::<Uuid>::None),
                )
                .finally(bookmark::Column::CollectionId.into_expr()),
                "collection_id",
            )
            .join(JoinType::Join, bookmark::Relation::Collection.def())
            .filter(collection::Column::ProfileId.eq(params.profile_id));

        if let Some(published_at) = params.published_at {
            query = query.filter(bookmark::Column::PublishedAt.lt(published_at))
        }
        if params.should_filter {
            if let Some(collection_id) = params.collection_id {
                query = query.filter(bookmark::Column::CollectionId.eq(collection_id))
            } else {
                query = query.filter(collection::Column::IsDefault.eq(true))
            }
        }

        query
            .order_by_desc(bookmark::Column::PublishedAt)
            .order_by_desc(bookmark::Column::OriginalPublishedAt)
            .order_by_asc(bookmark::Column::Title)
            .order_by_asc(bookmark::Column::OriginalTitle)
            .order_by_asc(collection::Column::Id)
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
                    let mut query = collection::Entity::find()
                        .select_only()
                        .column(collection::Column::Id);
                    if let Some(collection_id) = data.collection_id {
                        query = query.filter(collection::Column::Id.eq(collection_id));
                    } else {
                        query = query.filter(collection::Column::IsDefault.eq(true))
                    }

                    let Some(collection) = query
                        .filter(collection::Column::ProfileId.eq(data.profile_id))
                        .into_model::<CollectionSelect>()
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        if let Some(collection_id) = data.collection_id {
                            return Err(Error::Collection(
                                colette_core::collections::Error::NotFound(collection_id),
                            ));
                        } else {
                            return Err(Error::Collection(
                                colette_core::collections::Error::Unknown(anyhow!(
                                    "Failed to fetch default collection"
                                )),
                            ));
                        }
                    };

                    let bookmark_model = bookmark::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        link: Set(data.url.clone()),
                        original_title: Set(data.bookmark.title),
                        original_thumbnail_url: Set(data.bookmark.thumbnail.map(String::from)),
                        original_published_at: Set(data
                            .bookmark
                            .published
                            .map(DateTime::<FixedOffset>::from)),
                        original_author: Set(data.bookmark.author),
                        collection_id: Set(collection.id),
                        ..Default::default()
                    };

                    bookmark::Entity::insert(bookmark_model)
                        .on_conflict(
                            OnConflict::columns([
                                bookmark::Column::CollectionId,
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
                        .expr_as(
                            Expr::case(
                                collection::Column::IsDefault.eq(true),
                                Expr::value(Option::<Uuid>::None),
                            )
                            .finally(bookmark::Column::CollectionId.into_expr()),
                            "collection_id",
                        )
                        .join(JoinType::Join, bookmark::Relation::Collection.def())
                        .filter(bookmark::Column::CollectionId.eq(collection.id))
                        .filter(bookmark::Column::Link.eq(data.url))
                        .filter(collection::Column::ProfileId.eq(data.profile_id))
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
                    let Some(mut bookmark) = bookmark_by_id(params.id, params.profile_id)
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };

                    let mut model = bookmark::ActiveModel {
                        id: Set(params.id),
                        ..Default::default()
                    };
                    if data.title.is_some() {
                        model.title = Set(data.title);
                    }
                    if data.thumbnail_url.is_some() {
                        model.thumbnail_url = Set(data.thumbnail_url);
                    }
                    if data.published_at.is_some() {
                        model.published_at =
                            Set(data.published_at.map(DateTime::<FixedOffset>::from));
                    }
                    if data.author.is_some() {
                        model.author = Set(data.author);
                    }

                    let model = bookmark::Entity::update(model)
                        .exec(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

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

                    bookmark.original_title = model.original_title;
                    bookmark.original_thumbnail_url = model.original_thumbnail_url;
                    bookmark.original_published_at = model.original_published_at;
                    bookmark.original_author = model.original_author;
                    bookmark.updated_at = model.updated_at;

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
        self.db
            .transaction::<_, (), Error>(|txn| {
                Box::pin(async move {
                    let Some(bookmark) = bookmark::Entity::find_by_id(params.id)
                        .select_only()
                        .column(bookmark::Column::Id)
                        .join(JoinType::Join, bookmark::Relation::Collection.def())
                        .filter(collection::Column::ProfileId.eq(params.profile_id))
                        .into_model::<BookmarkDelete>()
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };

                    bookmark::Entity::delete_by_id(bookmark.id)
                        .exec(txn)
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

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
struct BookmarkSelect {
    id: Uuid,
    link: String,
    title: Option<String>,
    thumbnail_url: Option<String>,
    published_at: Option<DateTime<FixedOffset>>,
    author: Option<String>,
    original_title: String,
    original_thumbnail_url: Option<String>,
    original_published_at: Option<DateTime<FixedOffset>>,
    original_author: Option<String>,
    collection_id: Option<Uuid>,
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
            original_title: value.original_title,
            original_published_at: value.original_published_at.map(DateTime::<Utc>::from),
            original_author: value.original_author,
            original_thumbnail_url: value.original_thumbnail_url,
            collection_id: value.collection_id,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
        }
    }
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
struct CollectionSelect {
    id: Uuid,
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
struct BookmarkDelete {
    id: Uuid,
}

const BOOKMARK_COLUMNS: [bookmark::Column; 12] = [
    bookmark::Column::Id,
    bookmark::Column::Link,
    bookmark::Column::Title,
    bookmark::Column::ThumbnailUrl,
    bookmark::Column::PublishedAt,
    bookmark::Column::Author,
    bookmark::Column::OriginalTitle,
    bookmark::Column::OriginalThumbnailUrl,
    bookmark::Column::OriginalPublishedAt,
    bookmark::Column::OriginalAuthor,
    bookmark::Column::CreatedAt,
    bookmark::Column::UpdatedAt,
];

fn bookmark_by_id(id: Uuid, profile_id: Uuid) -> Selector<SelectModel<BookmarkSelect>> {
    bookmark::Entity::find_by_id(id)
        .select_only()
        .columns(BOOKMARK_COLUMNS)
        .expr_as(
            Expr::case(
                collection::Column::IsDefault.eq(true),
                Expr::value(Option::<Uuid>::None),
            )
            .finally(bookmark::Column::CollectionId.into_expr()),
            "collection_id",
        )
        .join(JoinType::Join, bookmark::Relation::Collection.def())
        .filter(collection::Column::ProfileId.eq(profile_id))
        .into_model::<BookmarkSelect>()
}
