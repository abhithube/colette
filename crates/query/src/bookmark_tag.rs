use std::fmt::Write;

use sea_query::{
    DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Order, Query, SelectStatement,
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

pub struct BookmarkTagSelect<T> {
    pub bookmark_ids: T,
}

impl<I: IntoIterator<Item = Uuid>> IntoSelect for BookmarkTagSelect<I> {
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

pub struct BookmarkTagById {
    pub id: Uuid,
    pub user_id: Uuid,
}

pub struct BookmarkTagInsert<I> {
    pub bookmark_id: Uuid,
    pub tags: I,
}

impl<I: IntoIterator<Item = BookmarkTagById>> IntoInsert for BookmarkTagInsert<I> {
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
            query.values_panic([self.bookmark_id.into(), tag.id.into(), tag.user_id.into()]);
        }

        query
    }
}

pub struct BookmarkTagDelete<T> {
    pub bookmark_id: Uuid,
    pub tag_ids: T,
}

impl<I: IntoIterator<Item = Uuid>> IntoDelete for BookmarkTagDelete<I> {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(BookmarkTag::Table)
            .and_where(Expr::col(BookmarkTag::BookmarkId).eq(self.bookmark_id))
            .and_where(Expr::col(BookmarkTag::TagId).is_not_in(self.tag_ids))
            .to_owned()
    }
}
