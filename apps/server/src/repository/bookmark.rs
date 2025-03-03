use colette_core::{
    Bookmark,
    bookmark::{
        BookmarkCreateData, BookmarkDateField, BookmarkFilter, BookmarkFindParams,
        BookmarkRepository, BookmarkScrapedData, BookmarkTextField, BookmarkUpdateData, Error,
    },
    common::IdParams,
};
use colette_model::{BookmarkWithTags, bookmark_tags, bookmarks, tags};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, IntoActiveModel, LoaderTrait, ModelTrait, QueryFilter, QueryOrder, QuerySelect,
    QueryTrait, TransactionTrait,
    prelude::Expr,
    sea_query::{Query, SimpleExpr},
};
use uuid::Uuid;

use super::common::{self, ToColumn, ToSql};

#[derive(Debug, Clone)]
pub struct SqliteBookmarkRepository {
    db: DatabaseConnection,
}

impl SqliteBookmarkRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl BookmarkRepository for SqliteBookmarkRepository {
    async fn find_bookmarks(&self, params: BookmarkFindParams) -> Result<Vec<Bookmark>, Error> {
        let mut query = bookmarks::Entity::find()
            .filter(bookmarks::Column::UserId.eq(params.user_id.to_string()))
            .apply_if(params.cursor, |query, cursor| {
                query.filter(bookmarks::Column::CreatedAt.gt(cursor.created_at.timestamp()))
            })
            .order_by_asc(bookmarks::Column::CreatedAt)
            .limit(params.limit.map(|e| e as u64));

        if let Some(filter) = params.filter {
            query = query.filter(filter.to_sql());
        } else {
            query = query
                .apply_if(params.id, |query, id| {
                    query.filter(bookmarks::Column::Id.eq(id.to_string()))
                })
                .apply_if(params.tags, |query, tags| {
                    query.filter(Expr::exists(
                        Query::select()
                            .expr(Expr::val(1))
                            .from(bookmark_tags::Entity)
                            .and_where(
                                Expr::col(bookmark_tags::Column::BookmarkId)
                                    .eq(Expr::col(bookmarks::Column::Id)),
                            )
                            .and_where(
                                bookmark_tags::Column::TagId
                                    .is_in(tags.into_iter().map(String::from).collect::<Vec<_>>()),
                            )
                            .to_owned(),
                    ))
                })
        }

        let bookmark_models = query.all(&self.db).await?;

        let tag_models = bookmark_models
            .load_many_to_many(
                tags::Entity::find().order_by_asc(tags::Column::Title),
                bookmark_tags::Entity,
                &self.db,
            )
            .await?;

        let bookmarks = bookmark_models
            .into_iter()
            .zip(tag_models.into_iter())
            .map(|(bookmark, tags)| BookmarkWithTags { bookmark, tags }.into())
            .collect();

        Ok(bookmarks)
    }

    async fn create_bookmark(&self, data: BookmarkCreateData) -> Result<Uuid, Error> {
        let tx = self.db.begin().await?;

        let id = Uuid::new_v4();
        let user_id = data.user_id.to_string();
        let bookmark = bookmarks::ActiveModel {
            id: ActiveValue::Set(id.into()),
            link: ActiveValue::Set(data.url.to_string()),
            title: ActiveValue::Set(data.title.clone()),
            thumbnail_url: ActiveValue::Set(data.thumbnail_url.map(Into::into)),
            published_at: ActiveValue::Set(data.published_at.map(|e| e.timestamp() as i32)),
            author: ActiveValue::Set(data.author),
            user_id: ActiveValue::Set(user_id.clone()),
            ..Default::default()
        };
        bookmark.insert(&tx).await.map_err(|e| match e {
            DbErr::RecordNotInserted => Error::Conflict(data.url),
            _ => Error::Database(e),
        })?;

        if let Some(tags) = data.tags {
            link_tags(&tx, tags, id, data.user_id).await?;
        }

        tx.commit().await?;

        Ok(id)
    }

    async fn update_bookmark(
        &self,
        params: IdParams,
        data: BookmarkUpdateData,
    ) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let Some(bookmark) = bookmarks::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if bookmark.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        let mut bookmark = bookmark.into_active_model();

        if let Some(title) = data.title {
            bookmark.title = ActiveValue::Set(title);
        }
        if let Some(thumbnail_url) = data.thumbnail_url {
            bookmark.thumbnail_url = ActiveValue::Set(thumbnail_url.map(Into::into));
        }
        if let Some(published_at) = data.published_at {
            bookmark.published_at = ActiveValue::Set(published_at.map(|e| e.timestamp() as i32));
        }
        if let Some(author) = data.author {
            bookmark.author = ActiveValue::Set(author);
        }
        if let Some(archived_path) = data.archived_path {
            bookmark.archived_path = ActiveValue::Set(archived_path);
        }

        if bookmark.is_changed() {
            bookmark.update(&tx).await?;
        }

        if let Some(tags) = data.tags {
            link_tags(&tx, tags, params.id, params.user_id).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn delete_bookmark(&self, params: IdParams) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let Some(bookmark) = bookmarks::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if bookmark.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        bookmark.delete(&tx).await?;

        tx.commit().await?;

        Ok(())
    }

    async fn save_scraped(&self, data: BookmarkScrapedData) -> Result<(), Error> {
        common::upsert_bookmark(
            &self.db,
            data.url,
            data.bookmark.title,
            data.bookmark.thumbnail,
            data.bookmark.published,
            data.bookmark.author,
            data.user_id,
        )
        .await?;

        Ok(())
    }
}

async fn link_tags(
    tx: &DatabaseTransaction,
    tags: Vec<Uuid>,
    bookmark_id: Uuid,
    user_id: Uuid,
) -> Result<(), DbErr> {
    let bookmark_id = bookmark_id.to_string();
    let user_id = user_id.to_string();
    let tag_ids = tags.iter().map(|e| e.to_string());

    bookmark_tags::Entity::delete_many()
        .filter(bookmark_tags::Column::TagId.is_not_in(tag_ids.clone()))
        .exec(tx)
        .await?;

    let models = tag_ids.map(|e| bookmark_tags::ActiveModel {
        bookmark_id: ActiveValue::Set(bookmark_id.clone()),
        tag_id: ActiveValue::Set(e),
        user_id: ActiveValue::Set(user_id.clone()),
        ..Default::default()
    });
    bookmark_tags::Entity::insert_many(models)
        .on_conflict_do_nothing()
        .exec(tx)
        .await?;

    Ok(())
}

impl ToColumn for BookmarkTextField {
    fn to_column(self) -> Expr {
        match self {
            Self::Link => Expr::col((bookmarks::Entity, bookmarks::Column::Link)),
            Self::Title => Expr::col((bookmarks::Entity, bookmarks::Column::Title)),
            Self::Author => Expr::col((bookmarks::Entity, bookmarks::Column::Author)),
            Self::Tag => Expr::col((tags::Entity, tags::Column::Title)),
        }
    }
}

impl ToColumn for BookmarkDateField {
    fn to_column(self) -> Expr {
        match self {
            Self::PublishedAt => Expr::col((bookmarks::Entity, bookmarks::Column::PublishedAt)),
            Self::CreatedAt => Expr::col((bookmarks::Entity, bookmarks::Column::CreatedAt)),
            Self::UpdatedAt => Expr::col((bookmarks::Entity, bookmarks::Column::UpdatedAt)),
        }
    }
}

impl ToSql for BookmarkFilter {
    fn to_sql(self) -> SimpleExpr {
        match self {
            BookmarkFilter::Text { field, op } => match field {
                BookmarkTextField::Tag => Expr::exists(
                    Query::select()
                        .expr(Expr::val(1))
                        .from(bookmark_tags::Entity)
                        .inner_join(
                            tags::Entity,
                            Expr::col((tags::Entity, tags::Column::Id)).eq(Expr::col((
                                bookmark_tags::Entity,
                                bookmark_tags::Column::TagId,
                            ))),
                        )
                        .and_where(
                            Expr::col((bookmark_tags::Entity, bookmark_tags::Column::BookmarkId))
                                .eq(Expr::col((bookmarks::Entity, bookmarks::Column::Id))),
                        )
                        .and_where((field.to_column(), op).to_sql())
                        .to_owned(),
                ),
                _ => (field.to_column(), op).to_sql(),
            },
            BookmarkFilter::Date { field, op } => (field.to_column(), op).to_sql(),
            BookmarkFilter::And(filters) => {
                let mut conditions = filters.into_iter().map(|e| e.to_sql()).collect::<Vec<_>>();
                let mut and = conditions.swap_remove(0);

                for condition in conditions {
                    and = and.and(condition)
                }

                and
            }
            BookmarkFilter::Or(filters) => {
                let mut conditions = filters.into_iter().map(|e| e.to_sql()).collect::<Vec<_>>();
                let mut or = conditions.swap_remove(0);

                for condition in conditions {
                    or = or.or(condition)
                }

                or
            }
            BookmarkFilter::Not(filter) => filter.to_sql().not(),
            _ => unreachable!(),
        }
    }
}
