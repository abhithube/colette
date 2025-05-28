use std::fmt::Write;

use chrono::{DateTime, Utc};
use colette_core::tag::{TagParams, TagType};
use sea_query::{
    Alias, Asterisk, DeleteStatement, Expr, Func, Iden, InsertStatement, OnConflict, Order, Query,
    SelectStatement,
};
use uuid::Uuid;

use crate::{
    IntoDelete, IntoInsert, IntoSelect, bookmark_tag::BookmarkTag,
    subscription_tag::SubscriptionTag,
};

const SUBSCRIPTION_COUNT: &str = "subscription_count";
const BOOKMARK_COUNT: &str = "bookmark_count";

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
            .column((Tag::Table, Asterisk))
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

        if self.with_subscription_count || self.tag_type == Some(TagType::Feeds) {
            query
                .expr_as(
                    Func::count(Expr::col((
                        SubscriptionTag::Table,
                        SubscriptionTag::SubscriptionId,
                    ))),
                    Alias::new(SUBSCRIPTION_COUNT),
                )
                .left_join(
                    SubscriptionTag::Table,
                    Expr::col((SubscriptionTag::Table, SubscriptionTag::TagId))
                        .eq(Expr::col((Tag::Table, Tag::Id))),
                );
        }

        if self.with_bookmark_count || self.tag_type == Some(TagType::Bookmarks) {
            query
                .expr_as(
                    Func::count(Expr::col((BookmarkTag::Table, BookmarkTag::BookmarkId))),
                    Alias::new(BOOKMARK_COUNT),
                )
                .left_join(
                    BookmarkTag::Table,
                    Expr::col((BookmarkTag::Table, BookmarkTag::TagId))
                        .eq(Expr::col((Tag::Table, Tag::Id))),
                );
        }

        if self.with_subscription_count || self.with_bookmark_count || self.tag_type.is_some() {
            query.group_by_col((Tag::Table, Tag::Id));
        }

        if let Some(tag_type) = self.tag_type {
            match tag_type {
                TagType::Feeds => {
                    query.and_having(Expr::col(Alias::new(SUBSCRIPTION_COUNT)).gt(Expr::val(0)));
                }
                TagType::Bookmarks => {
                    query.and_having(Expr::col(Alias::new(BOOKMARK_COUNT)).gt(Expr::val(0)));
                }
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
    pub user_id: Uuid,
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
            query
                .on_conflict(
                    OnConflict::columns([Tag::UserId, Tag::Title])
                        .update_column(Tag::UpdatedAt)
                        .to_owned(),
                )
                .returning_col(Tag::Id);
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
