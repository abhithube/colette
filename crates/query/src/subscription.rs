use std::fmt::Write;

use chrono::{DateTime, Utc};
use colette_core::subscription::SubscriptionParams;
use sea_query::{
    Alias, Asterisk, DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Order, Query,
    SelectStatement,
};
use uuid::Uuid;

use crate::{IntoDelete, IntoInsert, IntoSelect, feed::Feed, subscription_tag::SubscriptionTag};

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

impl IntoSelect for SubscriptionParams {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .column((Subscription::Table, Asterisk))
            .from(Subscription::Table)
            .apply_if(self.id, |query, id| {
                query.and_where(Expr::col((Subscription::Table, Subscription::Id)).eq(id));
            })
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(Expr::col((Subscription::Table, Subscription::UserId)).eq(user_id));
            })
            .apply_if(self.tags, |query, tags| {
                query.and_where(Expr::exists(
                    Query::select()
                        .expr(Expr::val("1"))
                        .from(SubscriptionTag::Table)
                        .and_where(
                            Expr::col((SubscriptionTag::Table, SubscriptionTag::SubscriptionId))
                                .eq(Expr::col((Subscription::Table, Subscription::Id))),
                        )
                        .and_where(
                            Expr::col((SubscriptionTag::Table, SubscriptionTag::TagId)).is_in(tags),
                        )
                        .to_owned(),
                ));
            })
            .apply_if(self.cursor, |query, (title, id)| {
                query.and_where(
                    Expr::tuple([
                        Expr::col(Subscription::Title).into(),
                        Expr::col(Subscription::Id).into(),
                    ])
                    .gt(Expr::tuple([Expr::value(title), Expr::value(id)])),
                );
            })
            .order_by((Subscription::Table, Subscription::Title), Order::Asc)
            .order_by((Subscription::Table, Subscription::Id), Order::Asc)
            .to_owned();

        if self.with_feed {
            query
                .columns([
                    (Feed::Table, Feed::Link),
                    (Feed::Table, Feed::XmlUrl),
                    (Feed::Table, Feed::Description),
                    (Feed::Table, Feed::RefreshedAt),
                ])
                .expr_as(
                    Expr::col((Feed::Table, Feed::Title)),
                    Alias::new("feed_title"),
                )
                .inner_join(
                    Feed::Table,
                    Expr::col((Feed::Table, Feed::Id))
                        .eq(Expr::col((Subscription::Table, Subscription::FeedId))),
                );
        }

        if let Some(limit) = self.limit {
            query.limit(limit);
        }

        query
    }
}

pub struct SubscriptionInsert<'a> {
    pub id: Uuid,
    pub title: &'a str,
    pub feed_id: Uuid,
    pub user_id: &'a str,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub upsert: bool,
}

impl IntoInsert for SubscriptionInsert<'_> {
    fn into_insert(self) -> InsertStatement {
        let mut query = Query::insert()
            .into_table(Subscription::Table)
            .columns([
                Subscription::Id,
                Subscription::Title,
                Subscription::FeedId,
                Subscription::UserId,
                Subscription::CreatedAt,
                Subscription::UpdatedAt,
            ])
            .values_panic([
                self.id.into(),
                self.title.into(),
                self.feed_id.into(),
                self.user_id.into(),
                self.created_at.into(),
                self.updated_at.into(),
            ])
            .to_owned();

        if self.upsert {
            query
                .on_conflict(
                    OnConflict::columns([Subscription::UserId, Subscription::FeedId])
                        .update_columns([Subscription::Title, Subscription::UpdatedAt])
                        .to_owned(),
                )
                .returning_col(Subscription::Id);
        } else {
            query.on_conflict(
                OnConflict::column(Subscription::Id)
                    .update_columns([Subscription::Title, Subscription::UpdatedAt])
                    .to_owned(),
            );
        }

        query
    }
}

pub struct SubscriptionDelete {
    pub id: Uuid,
}

impl IntoDelete for SubscriptionDelete {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(Subscription::Table)
            .and_where(Expr::col(Subscription::Id).eq(self.id))
            .to_owned()
    }
}
