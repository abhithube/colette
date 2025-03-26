use std::fmt::Write;

use chrono::{DateTime, Utc};
use colette_core::bookmark::{
    BookmarkDateField, BookmarkFilter, BookmarkParams, BookmarkTextField,
};
use sea_query::{
    Asterisk, DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Order, Query,
    SelectStatement, SimpleExpr, UpdateStatement,
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

impl IntoSelect for BookmarkParams {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .column(Asterisk)
            .from(Bookmark::Table)
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(Expr::col((Bookmark::Table, Bookmark::UserId)).eq(user_id));
            })
            .apply_if(self.cursor, |query, created_at| {
                query.and_where(
                    Expr::col((Bookmark::Table, Bookmark::CreatedAt)).lt(Expr::val(created_at)),
                );
            })
            .order_by((Bookmark::Table, Bookmark::CreatedAt), Order::Desc)
            .to_owned();

        if let Some(filter) = self.filter {
            query.and_where(filter.to_sql());
        } else {
            query
                .apply_if(self.id, |query, id| {
                    query.and_where(Expr::col((Bookmark::Table, Bookmark::Id)).eq(id));
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
                                Expr::col((BookmarkTag::Table, BookmarkTag::TagId)).is_in(tags),
                            )
                            .to_owned(),
                    ));
                });
        }

        if let Some(limit) = self.limit {
            query.limit(limit);
        }

        query
    }
}

pub struct BookmarkInsert<'a> {
    pub id: Uuid,
    pub link: &'a str,
    pub title: &'a str,
    pub thumbnail_url: Option<&'a str>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<&'a str>,
    pub archived_path: Option<&'a str>,
    pub user_id: &'a str,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub upsert: bool,
}

impl IntoInsert for BookmarkInsert<'_> {
    fn into_insert(self) -> InsertStatement {
        let mut query = Query::insert()
            .into_table(Bookmark::Table)
            .columns([
                Bookmark::Id,
                Bookmark::Link,
                Bookmark::Title,
                Bookmark::ThumbnailUrl,
                Bookmark::PublishedAt,
                Bookmark::Author,
                Bookmark::ArchivedPath,
                Bookmark::UserId,
                Bookmark::CreatedAt,
                Bookmark::UpdatedAt,
            ])
            .values_panic([
                self.id.into(),
                self.link.into(),
                self.title.into(),
                self.thumbnail_url.into(),
                self.published_at.into(),
                self.author.into(),
                self.archived_path.into(),
                self.user_id.into(),
                self.created_at.into(),
                self.updated_at.into(),
            ])
            .to_owned();

        if self.upsert {
            query
                .on_conflict(
                    OnConflict::columns([Bookmark::UserId, Bookmark::Link])
                        .update_columns([
                            Bookmark::Title,
                            Bookmark::ThumbnailUrl,
                            Bookmark::PublishedAt,
                            Bookmark::Author,
                            Bookmark::ArchivedPath,
                            Bookmark::UpdatedAt,
                        ])
                        .to_owned(),
                )
                .returning_col(Bookmark::Id);
        } else {
            query.on_conflict(
                OnConflict::columns([Bookmark::Id])
                    .update_columns([
                        Bookmark::Title,
                        Bookmark::ThumbnailUrl,
                        Bookmark::PublishedAt,
                        Bookmark::Author,
                        Bookmark::ArchivedPath,
                        Bookmark::UpdatedAt,
                    ])
                    .to_owned(),
            );
        }

        query
    }
}

pub struct BookmarkUpdate<'a> {
    pub id: Uuid,
    pub archived_path: Option<Option<&'a str>>,
    pub updated_at: DateTime<Utc>,
}

impl IntoUpdate for BookmarkUpdate<'_> {
    fn into_update(self) -> UpdateStatement {
        let mut query = Query::update()
            .table(Bookmark::Table)
            .value(Bookmark::UpdatedAt, self.updated_at)
            .and_where(Expr::col(Bookmark::Id).eq(self.id))
            .to_owned();

        if let Some(archived_path) = self.archived_path {
            query.value(Bookmark::ArchivedPath, archived_path);
        }

        query
    }
}

pub struct BookmarkDelete {
    pub id: Uuid,
}

impl IntoDelete for BookmarkDelete {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(Bookmark::Table)
            .and_where(Expr::col(Bookmark::Id).eq(self.id))
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
            Self::Text { field, op } => match field {
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
            Self::Date { field, op } => (field.to_column(), op).to_sql(),
            Self::And(filters) => {
                let mut conditions = filters.into_iter().map(|e| e.to_sql()).collect::<Vec<_>>();
                let mut and = conditions.swap_remove(0);

                for condition in conditions {
                    and = and.and(condition)
                }

                and
            }
            Self::Or(filters) => {
                let mut conditions = filters.into_iter().map(|e| e.to_sql()).collect::<Vec<_>>();
                let mut or = conditions.swap_remove(0);

                for condition in conditions {
                    or = or.or(condition)
                }

                or
            }
            Self::Not(filter) => filter.to_sql().not(),
            _ => unreachable!(),
        }
    }
}
