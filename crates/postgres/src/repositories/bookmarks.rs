use anyhow::anyhow;
use colette_core::{
    bookmarks::{
        BookmarksCreateData, BookmarksFindManyParams, BookmarksRepository, BookmarksUpdateData,
        Error,
    },
    common::{self, FindOneParams},
    Bookmark,
};
use colette_entities::{bookmarks, collections};
use sea_orm::{
    prelude::Expr, sea_query::OnConflict, ColumnTrait, DatabaseConnection, EntityTrait, JoinType,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, SelectModel, Selector, Set,
    TransactionTrait,
};
use sqlx::types::chrono::{DateTime, FixedOffset, Utc};
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
        let mut query = bookmarks::Entity::find()
            .select_only()
            .columns(BOOKMARK_COLUMNS)
            .expr_as(
                Expr::case(
                    collections::Column::IsDefault.eq(true),
                    Expr::value(Option::<Uuid>::None),
                )
                .finally(bookmarks::Column::CollectionId.into_expr()),
                "collection_id",
            )
            .join(JoinType::Join, bookmarks::Relation::Collections.def())
            .filter(collections::Column::ProfileId.eq(params.profile_id));

        if let Some(published_at) = params.published_at {
            query = query.filter(bookmarks::Column::PublishedAt.lt(published_at))
        }
        if params.should_filter {
            if let Some(collection_id) = params.collection_id {
                query = query.filter(bookmarks::Column::CollectionId.eq(collection_id))
            } else {
                query = query.filter(collections::Column::IsDefault.eq(true))
            }
        }

        query
            .order_by_desc(bookmarks::Column::CustomPublishedAt)
            .order_by_desc(bookmarks::Column::PublishedAt)
            .order_by_asc(bookmarks::Column::CustomTitle)
            .order_by_asc(bookmarks::Column::Title)
            .order_by_asc(collections::Column::Id)
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
                    let mut query = collections::Entity::find()
                        .select_only()
                        .column(collections::Column::Id);
                    if let Some(collection_id) = data.collection_id {
                        query = query.filter(collections::Column::Id.eq(collection_id));
                    } else {
                        query = query.filter(collections::Column::IsDefault.eq(true))
                    }

                    let Some(collection_model) = query
                        .filter(collections::Column::ProfileId.eq(data.profile_id))
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        if let Some(collection_id) = data.collection_id {
                            return Err(Error::NotFound(collection_id));
                        } else {
                            return Err(Error::Unknown(anyhow!(
                                "Failed to fetch default collection"
                            )));
                        }
                    };

                    let bookmark_model = bookmarks::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        link: Set(data.url.clone()),
                        title: Set(data.bookmark.title),
                        thumbnail_url: Set(data.bookmark.thumbnail.map(String::from)),
                        published_at: Set(data
                            .bookmark
                            .published
                            .map(DateTime::<FixedOffset>::from)),
                        author: Set(data.bookmark.author),
                        collection_id: Set(collection_model.id),
                        ..Default::default()
                    };

                    bookmarks::Entity::insert(bookmark_model)
                        .on_conflict(
                            OnConflict::columns([
                                bookmarks::Column::CollectionId,
                                bookmarks::Column::Link,
                            ])
                            .update_columns([
                                bookmarks::Column::Title,
                                bookmarks::Column::ThumbnailUrl,
                                bookmarks::Column::PublishedAt,
                                bookmarks::Column::Author,
                            ])
                            .to_owned(),
                        )
                        .exec_without_returning(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let Some(bookmark) = bookmarks::Entity::find()
                        .select_only()
                        .columns(BOOKMARK_COLUMNS)
                        .expr_as(
                            Expr::case(
                                collections::Column::IsDefault.eq(true),
                                Expr::value(Option::<Uuid>::None),
                            )
                            .finally(bookmarks::Column::CollectionId.into_expr()),
                            "collection_id",
                        )
                        .join(JoinType::Join, bookmarks::Relation::Collections.def())
                        .filter(bookmarks::Column::CollectionId.eq(collection_model.id))
                        .filter(bookmarks::Column::Link.eq(data.url))
                        .filter(collections::Column::ProfileId.eq(data.profile_id))
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
            .map_err(|e| Error::Unknown(e.into()))
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

                    let mut model = bookmarks::ActiveModel {
                        id: Set(params.id),
                        ..Default::default()
                    };
                    if data.custom_title.is_some() {
                        model.custom_title = Set(data.custom_title);
                    }
                    if data.custom_thumbnail_url.is_some() {
                        model.custom_thumbnail_url = Set(data.custom_thumbnail_url);
                    }
                    if data.custom_published_at.is_some() {
                        model.custom_published_at =
                            Set(data.custom_published_at.map(DateTime::<FixedOffset>::from));
                    }
                    if data.custom_author.is_some() {
                        model.custom_author = Set(data.custom_author);
                    }

                    let model = bookmarks::Entity::update(model)
                        .exec(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    bookmark.custom_title = model.custom_title;
                    bookmark.custom_thumbnail_url = model.custom_thumbnail_url;
                    bookmark.custom_published_at = model.custom_published_at;
                    bookmark.custom_author = model.custom_author;
                    bookmark.updated_at = model.updated_at;

                    Ok(bookmark.into())
                })
            })
            .await
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn delete(&self, params: common::FindOneParams) -> Result<(), Error> {
        self.db
            .transaction::<_, (), Error>(|txn| {
                Box::pin(async move {
                    let Some(bookmark) = bookmarks::Entity::find_by_id(params.id)
                        .select_only()
                        .column(bookmarks::Column::Id)
                        .join(JoinType::Join, bookmarks::Relation::Collections.def())
                        .filter(collections::Column::ProfileId.eq(params.profile_id))
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };

                    bookmarks::Entity::delete_by_id(bookmark.id)
                        .exec(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| Error::Unknown(e.into()))
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
    custom_title: Option<String>,
    custom_thumbnail_url: Option<String>,
    custom_published_at: Option<DateTime<FixedOffset>>,
    custom_author: Option<String>,
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
            custom_title: value.custom_title,
            custom_published_at: value.custom_published_at.map(DateTime::<Utc>::from),
            custom_author: value.custom_author,
            custom_thumbnail_url: value.custom_thumbnail_url,
            collection_id: value.collection_id,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
        }
    }
}

const BOOKMARK_COLUMNS: [bookmarks::Column; 12] = [
    bookmarks::Column::Id,
    bookmarks::Column::Link,
    bookmarks::Column::Title,
    bookmarks::Column::ThumbnailUrl,
    bookmarks::Column::PublishedAt,
    bookmarks::Column::Author,
    bookmarks::Column::CustomTitle,
    bookmarks::Column::CustomThumbnailUrl,
    bookmarks::Column::CustomPublishedAt,
    bookmarks::Column::CustomAuthor,
    bookmarks::Column::CreatedAt,
    bookmarks::Column::UpdatedAt,
];

fn bookmark_by_id(id: Uuid, profile_id: Uuid) -> Selector<SelectModel<BookmarkSelect>> {
    bookmarks::Entity::find_by_id(id)
        .select_only()
        .columns(BOOKMARK_COLUMNS)
        .expr_as(
            Expr::case(
                collections::Column::IsDefault.eq(true),
                Expr::value(Option::<Uuid>::None),
            )
            .finally(bookmarks::Column::CollectionId.into_expr()),
            "collection_id",
        )
        .join(JoinType::Join, bookmarks::Relation::Collections.def())
        .filter(collections::Column::ProfileId.eq(profile_id))
        .into_model::<BookmarkSelect>()
}
