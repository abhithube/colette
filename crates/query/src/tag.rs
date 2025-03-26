use std::fmt::Write;

use chrono::{DateTime, Utc};
use colette_core::tag::{TagParams, TagType};
use sea_query::{
    Alias, DeleteStatement, Expr, Func, Iden, InsertStatement, OnConflict, Order, Query,
    SelectStatement,
};
use uuid::Uuid;

use crate::{
    IntoDelete, IntoInsert, IntoSelect, bookmark_tag::BookmarkTag,
    subscription_tag::SubscriptionTag,
};

pub enum Tag {
    Table,
    Id,
    Title,
    UserId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for Tag {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "tags",
                Self::Id => "id",
                Self::Title => "title",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

impl IntoSelect for TagParams {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .columns([
                (Tag::Table, Tag::Id),
                (Tag::Table, Tag::Title),
                (Tag::Table, Tag::UserId),
                (Tag::Table, Tag::CreatedAt),
                (Tag::Table, Tag::UpdatedAt),
            ])
            .from(Tag::Table)
            .apply_if(self.ids, |query, ids| {
                query.and_where(Expr::col((Tag::Table, Tag::Id)).is_in(ids));
            })
            .apply_if(self.title, |query, title| {
                query.and_where(Expr::col((Tag::Table, Tag::Title)).eq(title));
            })
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(Expr::col((Tag::Table, Tag::UserId)).eq(user_id));
            })
            .apply_if(self.cursor, |query, title| {
                query.and_where(Expr::col((Tag::Table, Tag::Title)).gt(Expr::val(title)));
            })
            .order_by((Tag::Table, Tag::CreatedAt), Order::Asc)
            .to_owned();

        if self.with_counts {
            let feed_count = Alias::new("feed_count");
            let bookmark_count = Alias::new("bookmark_count");

            query
                .expr_as(
                    Func::count(Expr::col((
                        SubscriptionTag::Table,
                        SubscriptionTag::SubscriptionId,
                    ))),
                    feed_count.clone(),
                )
                .expr_as(
                    Func::count(Expr::col((BookmarkTag::Table, BookmarkTag::BookmarkId))),
                    bookmark_count.clone(),
                )
                .left_join(
                    SubscriptionTag::Table,
                    Expr::col((SubscriptionTag::Table, SubscriptionTag::TagId))
                        .eq(Expr::col((Tag::Table, Tag::Id))),
                )
                .left_join(
                    BookmarkTag::Table,
                    Expr::col((BookmarkTag::Table, BookmarkTag::TagId))
                        .eq(Expr::col((Tag::Table, Tag::Id))),
                )
                .group_by_col((Tag::Table, Tag::Id));

            match self.tag_type {
                TagType::Feeds => {
                    query.and_having(Expr::col(feed_count).gt(Expr::val(0)));
                }
                TagType::Bookmarks => {
                    query.and_having(Expr::col(bookmark_count).gt(Expr::val(0)));
                }
                _ => {}
            }
        }

        if let Some(limit) = self.limit {
            query.limit(limit);
        }

        query
    }
}

pub struct TagInsert<'a> {
    pub id: Uuid,
    pub title: &'a str,
    pub user_id: &'a str,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub upsert: bool,
}

impl IntoInsert for TagInsert<'_> {
    fn into_insert(self) -> InsertStatement {
        let mut query = Query::insert()
            .into_table(Tag::Table)
            .columns([
                Tag::Id,
                Tag::Title,
                Tag::UserId,
                Tag::CreatedAt,
                Tag::UpdatedAt,
            ])
            .values_panic([
                self.id.into(),
                self.title.into(),
                self.user_id.into(),
                self.created_at.into(),
                self.updated_at.into(),
            ])
            .to_owned();

        if self.upsert {
            query.on_conflict(
                OnConflict::columns([Tag::UserId, Tag::Title])
                    .update_column(Tag::UpdatedAt)
                    .to_owned(),
            );
        } else {
            query.on_conflict(
                OnConflict::column(Tag::Id)
                    .update_columns([Tag::Title, Tag::UpdatedAt])
                    .to_owned(),
            );
        }

        query
    }
}

pub struct TagDelete {
    pub id: Uuid,
}

impl IntoDelete for TagDelete {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(Tag::Table)
            .and_where(Expr::col(Tag::Id).eq(self.id))
            .to_owned()
    }
}
