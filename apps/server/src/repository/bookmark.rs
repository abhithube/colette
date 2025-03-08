use std::collections::HashMap;

use colette_core::{
    Bookmark,
    bookmark::{
        BookmarkById, BookmarkCreateParams, BookmarkDateField, BookmarkDeleteParams,
        BookmarkFilter, BookmarkFindByIdParams, BookmarkFindParams, BookmarkRepository,
        BookmarkScrapedParams, BookmarkTextField, BookmarkUpdateParams, Error,
    },
    common::Transaction,
};
use colette_model::{
    BookmarkRow, BookmarkRowWithTagRows, BookmarkTagRow, bookmark_tags, bookmarks, tags,
};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr, FromQueryResult,
    TransactionTrait,
    sea_query::{Expr, OnConflict, Order, Query, SimpleExpr},
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
        let mut query = Query::select()
            .columns([
                (bookmarks::Entity, bookmarks::Column::Id),
                (bookmarks::Entity, bookmarks::Column::Link),
                (bookmarks::Entity, bookmarks::Column::Title),
                (bookmarks::Entity, bookmarks::Column::ThumbnailUrl),
                (bookmarks::Entity, bookmarks::Column::PublishedAt),
                (bookmarks::Entity, bookmarks::Column::Author),
                (bookmarks::Entity, bookmarks::Column::ArchivedPath),
                (bookmarks::Entity, bookmarks::Column::UserId),
                (bookmarks::Entity, bookmarks::Column::CreatedAt),
                (bookmarks::Entity, bookmarks::Column::UpdatedAt),
            ])
            .from(bookmarks::Entity)
            .apply_if(params.user_id, |query, user_id| {
                query.and_where(
                    Expr::col((bookmarks::Entity, bookmarks::Column::UserId))
                        .eq(user_id.to_string()),
                );
            })
            .apply_if(params.cursor, |query, cursor| {
                query.and_where(
                    Expr::col((bookmarks::Entity, bookmarks::Column::CreatedAt))
                        .lt(Expr::val(cursor.created_at.timestamp())),
                );
            })
            .order_by(
                (bookmarks::Entity, bookmarks::Column::CreatedAt),
                Order::Desc,
            )
            .to_owned();

        if let Some(filter) = params.filter {
            query.and_where(filter.to_sql());
        } else {
            query
                .apply_if(params.id, |query, id| {
                    query.and_where(
                        Expr::col((bookmarks::Entity, bookmarks::Column::Id)).eq(id.to_string()),
                    );
                })
                .apply_if(params.tags, |query, tags| {
                    query.and_where(Expr::exists(
                        Query::select()
                            .expr(Expr::val(1))
                            .from(bookmark_tags::Entity)
                            .and_where(
                                Expr::col((
                                    bookmark_tags::Entity,
                                    bookmark_tags::Column::BookmarkId,
                                ))
                                .eq(Expr::col((bookmarks::Entity, bookmarks::Column::Id))),
                            )
                            .and_where(
                                Expr::col((bookmark_tags::Entity, bookmark_tags::Column::TagId))
                                    .is_in(tags.into_iter().map(String::from)),
                            )
                            .to_owned(),
                    ));
                });
        }

        if let Some(limit) = params.limit {
            query.limit(limit as u64);
        }

        let bookmark_rows =
            BookmarkRow::find_by_statement(self.db.get_database_backend().build(&query))
                .all(&self.db)
                .await?;

        let query = Query::select()
            .column((bookmark_tags::Entity, bookmark_tags::Column::BookmarkId))
            .columns([
                (tags::Entity, tags::Column::Id),
                (tags::Entity, tags::Column::Title),
                (tags::Entity, tags::Column::CreatedAt),
                (tags::Entity, tags::Column::UpdatedAt),
                (tags::Entity, tags::Column::UserId),
            ])
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
                    .is_in(bookmark_rows.iter().map(|e| e.id.as_str())),
            )
            .order_by((tags::Entity, tags::Column::Title), Order::Asc)
            .to_owned();

        let tag_rows =
            BookmarkTagRow::find_by_statement(self.db.get_database_backend().build(&query))
                .all(&self.db)
                .await?;

        let mut tag_row_map = HashMap::<String, Vec<BookmarkTagRow>>::new();

        for row in tag_rows {
            tag_row_map
                .entry(row.bookmark_id.clone())
                .or_default()
                .push(row);
        }

        let bookmarks = bookmark_rows
            .into_iter()
            .map(|bookmark| {
                BookmarkRowWithTagRows {
                    tags: tag_row_map.remove(&bookmark.id),
                    bookmark,
                }
                .into()
            })
            .collect();

        Ok(bookmarks)
    }

    async fn find_bookmark_by_id(
        &self,
        tx: &dyn Transaction,
        params: BookmarkFindByIdParams,
    ) -> Result<BookmarkById, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let query = Query::select()
            .column((bookmarks::Entity, bookmarks::Column::Id))
            .column((bookmarks::Entity, bookmarks::Column::UserId))
            .from(bookmarks::Entity)
            .and_where(
                Expr::col((bookmarks::Entity, bookmarks::Column::Id)).eq(params.id.to_string()),
            )
            .to_owned();

        let Some(result) = tx
            .query_one(self.db.get_database_backend().build(&query))
            .await?
        else {
            return Err(Error::NotFound(params.id));
        };

        Ok(BookmarkById {
            id: result
                .try_get_by_index::<String>(0)
                .unwrap()
                .parse()
                .unwrap(),
            user_id: result
                .try_get_by_index::<String>(1)
                .unwrap()
                .parse()
                .unwrap(),
        })
    }

    async fn create_bookmark(&self, params: BookmarkCreateParams) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let query = Query::insert()
            .into_table(bookmarks::Entity)
            .columns([
                bookmarks::Column::Id,
                bookmarks::Column::Link,
                bookmarks::Column::Title,
                bookmarks::Column::ThumbnailUrl,
                bookmarks::Column::PublishedAt,
                bookmarks::Column::Author,
                bookmarks::Column::UserId,
            ])
            .values_panic([
                params.id.to_string().into(),
                params.url.to_string().into(),
                params.title.into(),
                params.thumbnail_url.map(String::from).into(),
                params.published_at.map(|e| e.timestamp()).into(),
                params.author.into(),
                params.user_id.to_string().into(),
            ])
            .to_owned();

        tx.execute(self.db.get_database_backend().build(&query))
            .await
            .map_err(|e| match e {
                DbErr::RecordNotInserted => Error::Conflict(params.url),
                _ => Error::Database(e),
            })?;

        if let Some(tags) = params.tags {
            link_tags(&tx, tags, params.id, params.user_id).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn update_bookmark(
        &self,
        tx: Option<&dyn Transaction>,
        params: BookmarkUpdateParams,
    ) -> Result<(), Error> {
        let tx = tx.map(|e| e.as_any().downcast_ref::<DatabaseTransaction>().unwrap());

        if params.title.is_none()
            && params.thumbnail_url.is_none()
            && params.published_at.is_none()
            && params.author.is_none()
            && params.archived_path.is_none()
        {
            return Ok(());
        }

        let mut query = Query::update()
            .table(bookmarks::Entity)
            .and_where(Expr::col(bookmarks::Column::Id).eq(params.id.to_string()))
            .to_owned();

        if let Some(title) = params.title {
            query.value(bookmarks::Column::Title, title);
        }
        if let Some(thumbnail_url) = params.thumbnail_url {
            query.value(
                bookmarks::Column::ThumbnailUrl,
                thumbnail_url.map(String::from),
            );
        }
        if let Some(published_at) = params.published_at {
            query.value(
                bookmarks::Column::ThumbnailUrl,
                published_at.map(|e| e.timestamp()),
            );
        }
        if let Some(author) = params.author {
            query.value(bookmarks::Column::Author, author);
        }
        if let Some(archived_path) = params.archived_path {
            query.value(bookmarks::Column::ArchivedPath, archived_path);
        }

        let statement = self.db.get_database_backend().build(&query);

        if let Some(tx) = tx {
            tx.execute(statement).await?;
        } else {
            self.db.execute(statement).await?;
        }

        // if let Some(tags) = data.tags {
        //     link_tags(&tx, tags, params.id, params.user_id).await?;
        // }

        Ok(())
    }

    async fn delete_bookmark(
        &self,
        tx: &dyn Transaction,
        params: BookmarkDeleteParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let query = Query::delete()
            .from_table(bookmarks::Entity)
            .and_where(Expr::col(bookmarks::Column::Id).eq(params.id.to_string()))
            .to_owned();

        tx.execute(self.db.get_database_backend().build(&query))
            .await?;

        Ok(())
    }

    async fn save_scraped(&self, params: BookmarkScrapedParams) -> Result<(), Error> {
        common::upsert_bookmark(
            &self.db,
            params.url,
            params.bookmark.title,
            params.bookmark.thumbnail,
            params.bookmark.published,
            params.bookmark.author,
            params.user_id,
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

    let query = Query::delete()
        .from_table(bookmark_tags::Entity)
        .and_where(Expr::col(bookmark_tags::Column::BookmarkId).eq(bookmark_id.as_str()))
        .and_where(Expr::col(bookmark_tags::Column::TagId).is_not_in(tag_ids.clone()))
        .to_owned();

    tx.execute(tx.get_database_backend().build(&query)).await?;

    let mut query = Query::insert()
        .into_table(bookmark_tags::Entity)
        .columns([
            bookmark_tags::Column::BookmarkId,
            bookmark_tags::Column::TagId,
            bookmark_tags::Column::UserId,
        ])
        .on_conflict(
            OnConflict::columns([
                bookmark_tags::Column::BookmarkId,
                bookmark_tags::Column::TagId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .to_owned();

    for tag_id in tag_ids {
        query.values_panic([
            bookmark_id.clone().into(),
            tag_id.into(),
            user_id.clone().into(),
        ]);
    }

    tx.execute(tx.get_database_backend().build(&query)).await?;

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
