use std::fmt::Write;

use colette_core::tag::TagById;
use sea_query::{
    DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Order, Query, SelectStatement,
    SimpleExpr,
};
use uuid::Uuid;

use crate::{IntoDelete, IntoInsert, IntoSelect, tag::Tag};

pub enum BookmarkTag {
    Table,
    BookmarkId,
    TagId,
    UserId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for BookmarkTag {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "bookmark_tags",
                Self::BookmarkId => "bookmark_id",
                Self::TagId => "tag_id",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

pub struct BookmarkTagSelectMany<T> {
    pub bookmark_ids: T,
}

impl<V: Into<SimpleExpr>, I: IntoIterator<Item = V>> IntoSelect for BookmarkTagSelectMany<I> {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column((BookmarkTag::Table, BookmarkTag::BookmarkId))
            .columns([
                (Tag::Table, Tag::Id),
                (Tag::Table, Tag::Title),
                (Tag::Table, Tag::CreatedAt),
                (Tag::Table, Tag::UpdatedAt),
                (Tag::Table, Tag::UserId),
            ])
            .from(BookmarkTag::Table)
            .inner_join(
                Tag::Table,
                Expr::col((Tag::Table, Tag::Id))
                    .eq(Expr::col((BookmarkTag::Table, BookmarkTag::TagId))),
            )
            .and_where(
                Expr::col((BookmarkTag::Table, BookmarkTag::BookmarkId)).is_in(self.bookmark_ids),
            )
            .order_by((Tag::Table, Tag::Title), Order::Asc)
            .to_owned()
    }
}

pub struct BookmarkTagUpsert {
    pub bookmark_id: Uuid,
    pub tag_id: Uuid,
    pub user_id: Uuid,
}

impl IntoInsert for BookmarkTagUpsert {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(BookmarkTag::Table)
            .columns([
                BookmarkTag::BookmarkId,
                BookmarkTag::TagId,
                BookmarkTag::UserId,
            ])
            .values_panic([
                self.bookmark_id.to_string().into(),
                self.tag_id.to_string().into(),
                self.user_id.to_string().into(),
            ])
            .on_conflict(
                OnConflict::columns([BookmarkTag::BookmarkId, BookmarkTag::TagId])
                    .do_nothing()
                    .to_owned(),
            )
            .to_owned()
    }
}

pub struct BookmarkTagDeleteMany<T> {
    pub bookmark_id: Uuid,
    pub tag_ids: T,
}

impl<V: Into<SimpleExpr>, I: IntoIterator<Item = V>> IntoDelete for BookmarkTagDeleteMany<I> {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(BookmarkTag::Table)
            .and_where(Expr::col(BookmarkTag::BookmarkId).eq(self.bookmark_id.to_string()))
            .and_where(Expr::col(BookmarkTag::TagId).is_not_in(self.tag_ids))
            .to_owned()
    }
}

pub struct BookmarkTagUpsertMany {
    pub bookmark_id: Uuid,
    pub tags: Vec<TagById>,
}

impl IntoInsert for BookmarkTagUpsertMany {
    fn into_insert(self) -> InsertStatement {
        let mut query = Query::insert()
            .into_table(BookmarkTag::Table)
            .columns([
                BookmarkTag::BookmarkId,
                BookmarkTag::TagId,
                BookmarkTag::UserId,
            ])
            .on_conflict(
                OnConflict::columns([BookmarkTag::BookmarkId, BookmarkTag::TagId])
                    .do_nothing()
                    .to_owned(),
            )
            .to_owned();

        for tag in self.tags {
            query.values_panic([
                self.bookmark_id.to_string().into(),
                tag.id.to_string().into(),
                tag.user_id.to_string().into(),
            ]);
        }

        query
    }
}
