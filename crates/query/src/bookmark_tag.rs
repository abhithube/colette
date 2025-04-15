use std::fmt::Write;

use sea_query::{DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Query};
use uuid::Uuid;

use crate::{IntoDelete, IntoInsert};

pub enum BookmarkTag {
    Table,
    BookmarkId,
    TagId,
    UserId,
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
            }
        )
        .unwrap();
    }
}

pub struct BookmarkTagInsert<'a, I> {
    pub bookmark_id: Uuid,
    pub user_id: &'a str,
    pub tag_ids: I,
}

impl<I: IntoIterator<Item = Uuid>> IntoInsert for BookmarkTagInsert<'_, I> {
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

        for tag_id in self.tag_ids {
            query.values_panic([self.bookmark_id.into(), tag_id.into(), self.user_id.into()]);
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
        let mut query = Query::delete()
            .from_table(BookmarkTag::Table)
            .and_where(Expr::col(BookmarkTag::BookmarkId).eq(self.bookmark_id))
            .to_owned();

        let it = self.tag_ids.into_iter().collect::<Vec<_>>();
        if !it.is_empty() {
            query.and_where(Expr::col(BookmarkTag::TagId).is_not_in(it));
        }

        query
    }
}
