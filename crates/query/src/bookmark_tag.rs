use colette_core::tag::TagById;
use colette_model::{bookmark_tags, tags};
use sea_query::{
    DeleteStatement, Expr, InsertStatement, OnConflict, Order, Query, SelectStatement, SimpleExpr,
};
use uuid::Uuid;

use crate::{IntoDelete, IntoInsert, IntoSelect};

pub struct BookmarkTagSelectMany<T> {
    pub bookmark_ids: T,
}

impl<V: Into<SimpleExpr>, I: IntoIterator<Item = V>> IntoSelect for BookmarkTagSelectMany<I> {
    fn into_select(self) -> SelectStatement {
        Query::select()
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
                    .is_in(self.bookmark_ids),
            )
            .order_by((tags::Entity, tags::Column::Title), Order::Asc)
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
            .into_table(bookmark_tags::Entity)
            .columns([
                bookmark_tags::Column::BookmarkId,
                bookmark_tags::Column::TagId,
                bookmark_tags::Column::UserId,
            ])
            .values_panic([
                self.bookmark_id.to_string().into(),
                self.tag_id.to_string().into(),
                self.user_id.to_string().into(),
            ])
            .on_conflict(
                OnConflict::columns([
                    bookmark_tags::Column::BookmarkId,
                    bookmark_tags::Column::TagId,
                ])
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
            .from_table(bookmark_tags::Entity)
            .and_where(
                Expr::col(bookmark_tags::Column::BookmarkId).eq(self.bookmark_id.to_string()),
            )
            .and_where(Expr::col(bookmark_tags::Column::TagId).is_not_in(self.tag_ids))
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
