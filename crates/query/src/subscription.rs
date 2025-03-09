use std::fmt::Write;

use colette_core::subscription::{
    SubscriptionCreateParams, SubscriptionDeleteParams, SubscriptionFindByIdParams,
    SubscriptionFindParams, SubscriptionUpdateParams,
};
use sea_query::{
    Alias, DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Order, Query, SelectStatement,
    UpdateStatement,
};
use uuid::Uuid;

use crate::{
    IntoDelete, IntoInsert, IntoSelect, IntoUpdate, feed::Feed, subscription_tag::SubscriptionTag,
};

pub enum Subscription {
    Table,
    Id,
    Title,
    UserId,
    FeedId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for Subscription {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "subscriptions",
                Self::Id => "id",
                Self::Title => "title",
                Self::FeedId => "feed_id",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

impl IntoSelect for SubscriptionFindParams {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .columns([
                (Subscription::Table, Subscription::Id),
                (Subscription::Table, Subscription::Title),
                (Subscription::Table, Subscription::UserId),
                (Subscription::Table, Subscription::CreatedAt),
                (Subscription::Table, Subscription::UpdatedAt),
            ])
            .columns([
                (Feed::Table, Feed::Link),
                (Feed::Table, Feed::XmlUrl),
                (Feed::Table, Feed::Description),
                (Feed::Table, Feed::RefreshedAt),
            ])
            .expr_as(Expr::col((Feed::Table, Feed::Id)), Alias::new("feed_id"))
            .expr_as(
                Expr::col((Feed::Table, Feed::Title)),
                Alias::new("feed_title"),
            )
            .from(Subscription::Table)
            .inner_join(
                Feed::Table,
                Expr::col((Feed::Table, Feed::Id))
                    .eq(Expr::col((Subscription::Table, Subscription::FeedId))),
            )
            .apply_if(self.id, |query, id| {
                query.and_where(
                    Expr::col((Subscription::Table, Subscription::Id)).eq(id.to_string()),
                );
            })
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(
                    Expr::col((Subscription::Table, Subscription::UserId)).eq(user_id.to_string()),
                );
            })
            .apply_if(self.tags, |query, tags| {
                query.and_where(Expr::exists(
                    Query::select()
                        .expr(Expr::val(1))
                        .from(SubscriptionTag::Table)
                        .and_where(
                            Expr::col((SubscriptionTag::Table, SubscriptionTag::SubscriptionId))
                                .eq(Expr::col((Subscription::Table, Subscription::Id))),
                        )
                        .and_where(
                            Expr::col((SubscriptionTag::Table, SubscriptionTag::TagId))
                                .is_in(tags.into_iter().map(String::from)),
                        )
                        .to_owned(),
                ));
            })
            .apply_if(self.cursor, |query, cursor| {
                query.and_where(
                    Expr::tuple([
                        Expr::col(Subscription::Title).into(),
                        Expr::col(Subscription::Id).into(),
                    ])
                    .gt(Expr::tuple([
                        Expr::value(cursor.title),
                        Expr::value(cursor.id.to_string()),
                    ])),
                );
            })
            .order_by((Subscription::Table, Subscription::Title), Order::Asc)
            .order_by((Subscription::Table, Subscription::Id), Order::Asc)
            .to_owned();

        if let Some(limit) = self.limit {
            query.limit(limit as u64);
        }

        query
    }
}

impl IntoSelect for SubscriptionFindByIdParams {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column((Subscription::Table, Subscription::Id))
            .column((Subscription::Table, Subscription::UserId))
            .from(Subscription::Table)
            .and_where(Expr::col((Subscription::Table, Subscription::Id)).eq(self.id.to_string()))
            .to_owned()
    }
}

impl IntoInsert for SubscriptionCreateParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(Subscription::Table)
            .columns([
                Subscription::Id,
                Subscription::Title,
                Subscription::FeedId,
                Subscription::UserId,
            ])
            .values_panic([
                self.id.to_string().into(),
                self.title.clone().into(),
                self.feed_id.to_string().into(),
                self.user_id.to_string().into(),
            ])
            .to_owned()
    }
}

impl IntoUpdate for SubscriptionUpdateParams {
    fn into_update(self) -> UpdateStatement {
        let mut query = Query::update()
            .table(Subscription::Table)
            .and_where(Expr::col(Subscription::Id).eq(self.id.to_string()))
            .to_owned();

        if let Some(title) = self.title {
            query.value(Subscription::Title, title);
        }

        query
    }
}

impl IntoDelete for SubscriptionDeleteParams {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(Subscription::Table)
            .and_where(Expr::col(Subscription::Id).eq(self.id.to_string()))
            .to_owned()
    }
}

pub struct SubscriptionUpsert {
    pub id: Uuid,
    pub title: String,
    pub feed_id: Uuid,
    pub user_id: Uuid,
}

impl IntoInsert for SubscriptionUpsert {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(Subscription::Table)
            .columns([
                Subscription::Id,
                Subscription::Title,
                Subscription::FeedId,
                Subscription::UserId,
            ])
            .values_panic([
                self.id.to_string().into(),
                self.title.into(),
                self.feed_id.to_string().into(),
                self.user_id.to_string().into(),
            ])
            .on_conflict(
                OnConflict::columns([Subscription::UserId, Subscription::FeedId])
                    .update_column(Subscription::Title)
                    .to_owned(),
            )
            .returning_col(Subscription::Id)
            .to_owned()
    }
}
