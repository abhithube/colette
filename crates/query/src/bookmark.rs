use std::fmt::Write;

use colette_core::bookmark::{
    BookmarkCreateParams, BookmarkDateField, BookmarkDeleteParams, BookmarkFilter,
    BookmarkFindByIdParams, BookmarkFindParams, BookmarkTextField, BookmarkUpdateParams,
    BookmarkUpsertParams,
};
use sea_query::{
    DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Order, Query, SelectStatement,
    SimpleExpr, UpdateStatement,
};
use uuid::Uuid;

use crate::{
    IntoDelete, IntoInsert, IntoSelect, IntoUpdate,
    bookmark_tag::BookmarkTag,
    filter::{ToColumn, ToSql},
    tag::Tag,
};

pub enum Bookmark {
    Table,
    Id,
    Link,
    Title,
    ThumbnailUrl,
    PublishedAt,
    Author,
    ArchivedPath,
    UserId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for Bookmark {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "bookmarks",
                Self::Id => "id",
                Self::Link => "link",
                Self::Title => "title",
                Self::ThumbnailUrl => "thumbnail_url",
                Self::PublishedAt => "published_at",
                Self::Author => "author",
                Self::ArchivedPath => "archived_path",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

impl IntoSelect for BookmarkFindParams {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .columns([
                (Bookmark::Table, Bookmark::Id),
                (Bookmark::Table, Bookmark::Link),
                (Bookmark::Table, Bookmark::Title),
                (Bookmark::Table, Bookmark::ThumbnailUrl),
                (Bookmark::Table, Bookmark::PublishedAt),
                (Bookmark::Table, Bookmark::Author),
                (Bookmark::Table, Bookmark::ArchivedPath),
                (Bookmark::Table, Bookmark::UserId),
                (Bookmark::Table, Bookmark::CreatedAt),
                (Bookmark::Table, Bookmark::UpdatedAt),
            ])
            .from(Bookmark::Table)
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(
                    Expr::col((Bookmark::Table, Bookmark::UserId)).eq(user_id.to_string()),
                );
            })
            .apply_if(self.cursor, |query, cursor| {
                query.and_where(
                    Expr::col((Bookmark::Table, Bookmark::CreatedAt))
                        .lt(Expr::val(cursor.created_at.timestamp())),
                );
            })
            .order_by((Bookmark::Table, Bookmark::CreatedAt), Order::Desc)
            .to_owned();

        if let Some(filter) = self.filter {
            query.and_where(filter.to_sql());
        } else {
            query
                .apply_if(self.id, |query, id| {
                    query.and_where(Expr::col((Bookmark::Table, Bookmark::Id)).eq(id.to_string()));
                })
                .apply_if(self.tags, |query, tags| {
                    query.and_where(Expr::exists(
                        Query::select()
                            .expr(Expr::val(1))
                            .from(BookmarkTag::Table)
                            .and_where(
                                Expr::col((BookmarkTag::Table, BookmarkTag::BookmarkId))
                                    .eq(Expr::col((Bookmark::Table, Bookmark::Id))),
                            )
                            .and_where(
                                Expr::col((BookmarkTag::Table, BookmarkTag::TagId))
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
            .column((Bookmark::Table, Bookmark::Id))
            .column((Bookmark::Table, Bookmark::UserId))
            .from(Bookmark::Table)
            .and_where(Expr::col((Bookmark::Table, Bookmark::Id)).eq(self.id.to_string()))
            .to_owned()
    }
}

impl IntoInsert for BookmarkCreateParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(Bookmark::Table)
            .columns([
                Bookmark::Id,
                Bookmark::Link,
                Bookmark::Title,
                Bookmark::ThumbnailUrl,
                Bookmark::PublishedAt,
                Bookmark::Author,
                Bookmark::UserId,
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
            .table(Bookmark::Table)
            .and_where(Expr::col(Bookmark::Id).eq(self.id.to_string()))
            .to_owned();

        if let Some(title) = self.title {
            query.value(Bookmark::Title, title);
        }
        if let Some(thumbnail_url) = self.thumbnail_url {
            query.value(Bookmark::ThumbnailUrl, thumbnail_url.map(String::from));
        }
        if let Some(published_at) = self.published_at {
            query.value(Bookmark::PublishedAt, published_at.map(|e| e.timestamp()));
        }
        if let Some(author) = self.author {
            query.value(Bookmark::Author, author);
        }
        if let Some(archived_path) = self.archived_path {
            query.value(Bookmark::ArchivedPath, archived_path);
        }

        query
    }
}

impl IntoDelete for BookmarkDeleteParams {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(Bookmark::Table)
            .and_where(Expr::col(Bookmark::Id).eq(self.id.to_string()))
            .to_owned()
    }
}

impl IntoInsert for BookmarkUpsertParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(Bookmark::Table)
            .columns([
                Bookmark::Id,
                Bookmark::Link,
                Bookmark::Title,
                Bookmark::ThumbnailUrl,
                Bookmark::PublishedAt,
                Bookmark::Author,
                Bookmark::UserId,
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
                OnConflict::columns([Bookmark::UserId, Bookmark::Link])
                    .update_columns([
                        Bookmark::Title,
                        Bookmark::ThumbnailUrl,
                        Bookmark::PublishedAt,
                        Bookmark::Author,
                    ])
                    .to_owned(),
            )
            .returning_col(Bookmark::Id)
            .to_owned()
    }
}

impl ToColumn for BookmarkTextField {
    fn to_column(self) -> Expr {
        match self {
            Self::Link => Expr::col((Bookmark::Table, Bookmark::Link)),
            Self::Title => Expr::col((Bookmark::Table, Bookmark::Title)),
            Self::Author => Expr::col((Bookmark::Table, Bookmark::Author)),
            Self::Tag => Expr::col((Tag::Table, Tag::Title)),
        }
    }
}

impl ToColumn for BookmarkDateField {
    fn to_column(self) -> Expr {
        match self {
            Self::PublishedAt => Expr::col((Bookmark::Table, Bookmark::PublishedAt)),
            Self::CreatedAt => Expr::col((Bookmark::Table, Bookmark::CreatedAt)),
            Self::UpdatedAt => Expr::col((Bookmark::Table, Bookmark::UpdatedAt)),
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
                        .from(BookmarkTag::Table)
                        .inner_join(
                            Tag::Table,
                            Expr::col((Tag::Table, Tag::Id))
                                .eq(Expr::col((BookmarkTag::Table, BookmarkTag::TagId))),
                        )
                        .and_where(
                            Expr::col((BookmarkTag::Table, BookmarkTag::BookmarkId))
                                .eq(Expr::col((Bookmark::Table, Bookmark::Id))),
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
