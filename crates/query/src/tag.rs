use colette_core::tag::{
    TagCreateParams, TagDeleteParams, TagFindByIdsParams, TagFindParams, TagType, TagUpdateParams,
};
use colette_model::{bookmark_tags, subscription_tags, tags};
use sea_query::{
    Alias, DeleteStatement, Expr, Func, InsertStatement, Order, Query, SelectStatement,
    UpdateStatement,
};
use uuid::Uuid;

use crate::{IntoDelete, IntoInsert, IntoSelect, IntoUpdate};

impl IntoSelect for TagFindParams {
    fn into_select(self) -> SelectStatement {
        let feed_count = Alias::new("feed_count");
        let bookmark_count = Alias::new("bookmark_count");

        let mut query = Query::select()
            .columns([
                (tags::Entity, tags::Column::Id),
                (tags::Entity, tags::Column::Title),
                (tags::Entity, tags::Column::UserId),
                (tags::Entity, tags::Column::CreatedAt),
                (tags::Entity, tags::Column::UpdatedAt),
            ])
            .expr_as(
                Func::count(Expr::col((
                    subscription_tags::Entity,
                    subscription_tags::Column::SubscriptionId,
                ))),
                feed_count.clone(),
            )
            .expr_as(
                Func::count(Expr::col((
                    bookmark_tags::Entity,
                    bookmark_tags::Column::BookmarkId,
                ))),
                bookmark_count.clone(),
            )
            .from(tags::Entity)
            .left_join(
                subscription_tags::Entity,
                Expr::col((subscription_tags::Entity, subscription_tags::Column::TagId))
                    .eq(Expr::col((tags::Entity, tags::Column::Id))),
            )
            .left_join(
                bookmark_tags::Entity,
                Expr::col((bookmark_tags::Entity, bookmark_tags::Column::TagId))
                    .eq(Expr::col((tags::Entity, tags::Column::Id))),
            )
            .apply_if(self.ids, |query, ids| {
                query.and_where(
                    Expr::col((tags::Entity, tags::Column::Id))
                        .is_in(ids.into_iter().map(String::from)),
                );
            })
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(
                    Expr::col((tags::Entity, tags::Column::UserId)).eq(user_id.to_string()),
                );
            })
            .apply_if(self.cursor, |query, cursor| {
                query.and_where(
                    Expr::col((tags::Entity, tags::Column::Title)).gt(Expr::val(cursor.title)),
                );
            })
            .group_by_col((tags::Entity, tags::Column::Id))
            .order_by((tags::Entity, tags::Column::CreatedAt), Order::Asc)
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
            .column((tags::Entity, tags::Column::Id))
            .column((tags::Entity, tags::Column::UserId))
            .from(tags::Entity)
            .and_where(
                Expr::col((tags::Entity, tags::Column::Id))
                    .is_in(self.ids.into_iter().map(String::from)),
            )
            .to_owned()
    }
}

impl IntoInsert for TagCreateParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .columns([tags::Column::Id, tags::Column::Title, tags::Column::UserId])
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
            .table(tags::Entity)
            .and_where(Expr::col(tags::Column::Id).eq(self.id.to_string()))
            .to_owned();

        if let Some(title) = self.title {
            query.value(tags::Column::Title, title);
        }

        query
    }
}

impl IntoDelete for TagDeleteParams {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(tags::Entity)
            .and_where(Expr::col(tags::Column::Id).eq(self.id.to_string()))
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
            .column(tags::Column::Id)
            .from(tags::Entity)
            .and_where(Expr::col(tags::Column::UserId).eq(self.user_id.to_string()))
            .and_where(Expr::col(tags::Column::UserId).eq(self.title.clone()))
            .to_owned()
    }
}

impl IntoInsert for TagUpsert {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(tags::Entity)
            .columns([tags::Column::Id, tags::Column::Title, tags::Column::UserId])
            .values_panic([
                self.id.to_string().into(),
                self.title.into(),
                self.user_id.to_string().into(),
            ])
            .to_owned()
    }
}
