use colette_core::bookmark::{
    BookmarkCreateParams, BookmarkDateField, BookmarkDeleteParams, BookmarkFilter,
    BookmarkFindByIdParams, BookmarkFindParams, BookmarkTextField, BookmarkUpdateParams,
    BookmarkUpsertParams,
};
use colette_model::{bookmark_tags, bookmarks, tags};
use sea_query::{
    DeleteStatement, Expr, InsertStatement, OnConflict, Order, Query, SelectStatement, SimpleExpr,
    UpdateStatement,
};
use uuid::Uuid;

use crate::{
    IntoDelete, IntoInsert, IntoSelect, IntoUpdate,
    filter::{ToColumn, ToSql},
};

impl IntoSelect for BookmarkFindParams {
    fn into_select(self) -> SelectStatement {
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
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(
                    Expr::col((bookmarks::Entity, bookmarks::Column::UserId))
                        .eq(user_id.to_string()),
                );
            })
            .apply_if(self.cursor, |query, cursor| {
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

        if let Some(filter) = self.filter {
            query.and_where(filter.to_sql());
        } else {
            query
                .apply_if(self.id, |query, id| {
                    query.and_where(
                        Expr::col((bookmarks::Entity, bookmarks::Column::Id)).eq(id.to_string()),
                    );
                })
                .apply_if(self.tags, |query, tags| {
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

        if let Some(limit) = self.limit {
            query.limit(limit as u64);
        }

        query
    }
}

impl IntoSelect for BookmarkFindByIdParams {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column((bookmarks::Entity, bookmarks::Column::Id))
            .column((bookmarks::Entity, bookmarks::Column::UserId))
            .from(bookmarks::Entity)
            .and_where(
                Expr::col((bookmarks::Entity, bookmarks::Column::Id)).eq(self.id.to_string()),
            )
            .to_owned()
    }
}

impl IntoInsert for BookmarkCreateParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
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
                self.id.to_string().into(),
                self.url.to_string().into(),
                self.title.into(),
                self.thumbnail_url.map(String::from).into(),
                self.published_at.map(|e| e.timestamp()).into(),
                self.author.into(),
                self.user_id.to_string().into(),
            ])
            .to_owned()
    }
}

impl IntoUpdate for BookmarkUpdateParams {
    fn into_update(self) -> UpdateStatement {
        let mut query = Query::update()
            .table(bookmarks::Entity)
            .and_where(Expr::col(bookmarks::Column::Id).eq(self.id.to_string()))
            .to_owned();

        if let Some(title) = self.title {
            query.value(bookmarks::Column::Title, title);
        }
        if let Some(thumbnail_url) = self.thumbnail_url {
            query.value(
                bookmarks::Column::ThumbnailUrl,
                thumbnail_url.map(String::from),
            );
        }
        if let Some(published_at) = self.published_at {
            query.value(
                bookmarks::Column::ThumbnailUrl,
                published_at.map(|e| e.timestamp()),
            );
        }
        if let Some(author) = self.author {
            query.value(bookmarks::Column::Author, author);
        }
        if let Some(archived_path) = self.archived_path {
            query.value(bookmarks::Column::ArchivedPath, archived_path);
        }

        query
    }
}

impl IntoDelete for BookmarkDeleteParams {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(bookmarks::Entity)
            .and_where(Expr::col(bookmarks::Column::Id).eq(self.id.to_string()))
            .to_owned()
    }
}

impl IntoInsert for BookmarkUpsertParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
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
                Uuid::new_v4().to_string().into(),
                self.url.to_string().into(),
                self.bookmark.title.into(),
                self.bookmark.thumbnail.map(String::from).into(),
                self.bookmark.published.map(|e| e.timestamp()).into(),
                self.bookmark.author.into(),
                self.user_id.to_string().into(),
            ])
            .on_conflict(
                OnConflict::columns([bookmarks::Column::UserId, bookmarks::Column::Link])
                    .update_columns([
                        bookmarks::Column::Title,
                        bookmarks::Column::ThumbnailUrl,
                        bookmarks::Column::PublishedAt,
                        bookmarks::Column::Author,
                    ])
                    .to_owned(),
            )
            .returning_col(bookmarks::Column::Id)
            .to_owned()
    }
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
