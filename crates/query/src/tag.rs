use std::fmt::Write;

use colette_core::tag::{
    TagCreateParams, TagDeleteParams, TagFindByIdsParams, TagFindParams, TagType, TagUpdateParams,
};
use sea_query::{
    Alias, DeleteStatement, Expr, Func, Iden, InsertStatement, Order, Query, SelectStatement,
    UpdateStatement,
};
use uuid::Uuid;

use crate::{
    IntoDelete, IntoInsert, IntoSelect, IntoUpdate, bookmark_tag::BookmarkTag,
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

impl IntoSelect for TagFindParams {
    fn into_select(self) -> SelectStatement {
        let feed_count = Alias::new("feed_count");
        let bookmark_count = Alias::new("bookmark_count");

        let mut query = Query::select()
            .columns([
                (Tag::Table, Tag::Id),
                (Tag::Table, Tag::Title),
                (Tag::Table, Tag::UserId),
                (Tag::Table, Tag::CreatedAt),
                (Tag::Table, Tag::UpdatedAt),
            ])
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
            .from(Tag::Table)
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
            .apply_if(self.ids, |query, ids| {
                query.and_where(
                    Expr::col((Tag::Table, Tag::Id)).is_in(ids.into_iter().map(String::from)),
                );
            })
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(Expr::col((Tag::Table, Tag::UserId)).eq(user_id.to_string()));
            })
            .apply_if(self.cursor, |query, cursor| {
                query.and_where(Expr::col((Tag::Table, Tag::Title)).gt(Expr::val(cursor.title)));
            })
            .group_by_col((Tag::Table, Tag::Id))
            .order_by((Tag::Table, Tag::CreatedAt), Order::Asc)
            .to_owned();

        match self.tag_type {
            TagType::Feeds => {
                query.and_having(Expr::col(feed_count).gt(Expr::val(0)));
            }
            TagType::Bookmarks => {
                query.and_having(Expr::col(bookmark_count).gt(Expr::val(0)));
            }
            _ => {}
        }

        if let Some(limit) = self.limit {
            query.limit(limit as u64);
        }

        query
    }
}

impl IntoSelect for TagFindByIdsParams {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column((Tag::Table, Tag::Id))
            .column((Tag::Table, Tag::UserId))
            .from(Tag::Table)
            .and_where(
                Expr::col((Tag::Table, Tag::Id)).is_in(self.ids.into_iter().map(String::from)),
            )
            .to_owned()
    }
}

impl IntoInsert for TagCreateParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .columns([Tag::Id, Tag::Title, Tag::UserId])
            .values_panic([
                self.id.to_string().into(),
                self.title.clone().into(),
                self.user_id.to_string().into(),
            ])
            .to_owned()
    }
}

impl IntoUpdate for TagUpdateParams {
    fn into_update(self) -> UpdateStatement {
        let mut query = Query::update()
            .table(Tag::Table)
            .and_where(Expr::col(Tag::Id).eq(self.id.to_string()))
            .to_owned();

        if let Some(title) = self.title {
            query.value(Tag::Title, title);
        }

        query
    }
}

impl IntoDelete for TagDeleteParams {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(Tag::Table)
            .and_where(Expr::col(Tag::Id).eq(self.id.to_string()))
            .to_owned()
    }
}

#[derive(Clone)]
pub struct TagUpsert {
    pub id: Uuid,
    pub title: String,
    pub user_id: Uuid,
}

impl IntoSelect for TagUpsert {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column(Tag::Id)
            .from(Tag::Table)
            .and_where(Expr::col(Tag::UserId).eq(self.user_id.to_string()))
            .and_where(Expr::col(Tag::UserId).eq(self.title.clone()))
            .to_owned()
    }
}

impl IntoInsert for TagUpsert {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(Tag::Table)
            .columns([Tag::Id, Tag::Title, Tag::UserId])
            .values_panic([
                self.id.to_string().into(),
                self.title.into(),
                self.user_id.to_string().into(),
            ])
            .to_owned()
    }
}
