use std::fmt::Write;

use colette_core::tag::{TagType, TagUpsertType};
use sea_query::{
    Alias, Asterisk, DeleteStatement, Expr, Func, Iden, InsertStatement, OnConflict, Order, Query,
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

pub struct TagSelect<'a, I> {
    pub ids: Option<I>,
    pub tag_type: TagType,
    pub feed_id: Option<Uuid>,
    pub bookmark_id: Option<Uuid>,
    pub user_id: Option<&'a str>,
    pub cursor: Option<&'a str>,
    pub limit: Option<u64>,
}

impl<I: IntoIterator<Item = Uuid>> IntoSelect for TagSelect<'_, I> {
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
                query.and_where(Expr::col((Tag::Table, Tag::Id)).is_in(ids));
            })
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(Expr::col((Tag::Table, Tag::UserId)).eq(user_id));
            })
            .apply_if(self.cursor, |query, title| {
                query.and_where(Expr::col((Tag::Table, Tag::Title)).gt(Expr::val(title)));
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
            query.limit(limit);
        }

        query
    }
}

pub enum TagSelectOne<'a> {
    Id(Uuid),
    Ids(Vec<Uuid>),
    Index { title: &'a str, user_id: &'a str },
}

impl IntoSelect for TagSelectOne<'_> {
    fn into_select(self) -> SelectStatement {
        let r#where = match self {
            Self::Id(id) => Expr::col(Tag::Id).eq(id),
            Self::Ids(ids) => Expr::col(Tag::Id).is_in(ids),
            Self::Index { title, user_id } => Expr::col(Tag::UserId)
                .eq(user_id)
                .and(Expr::col(Tag::Title).eq(title)),
        };

        Query::select()
            .column(Asterisk)
            .from(Tag::Table)
            .and_where(r#where)
            .to_owned()
    }
}

pub struct TagInsert<'a> {
    pub id: Uuid,
    pub title: &'a str,
    pub user_id: &'a str,
    pub upsert: Option<TagUpsertType>,
}

impl IntoInsert for TagInsert<'_> {
    fn into_insert(self) -> InsertStatement {
        let mut query = Query::insert()
            .into_table(Tag::Table)
            .columns([Tag::Id, Tag::Title, Tag::UserId])
            .values_panic([self.id.into(), self.title.into(), self.user_id.into()])
            .to_owned();

        if let Some(upsert) = self.upsert {
            let mut on_conflict = match upsert {
                TagUpsertType::Id => OnConflict::column(Tag::Id)
                    .update_column(Tag::Title)
                    .to_owned(),
                TagUpsertType::Title => OnConflict::columns([Tag::UserId, Tag::Title])
                    .do_nothing()
                    .to_owned(),
            };

            query.on_conflict(
                on_conflict
                    .value(Tag::UpdatedAt, Expr::current_timestamp())
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
