use chrono::{DateTime, Utc};
use colette_core::feed::{FeedFindParams, FeedStreamUrlsParams};
use colette_model::feeds;
use sea_query::{Asterisk, Expr, Func, InsertStatement, OnConflict, Order, Query, SelectStatement};
use url::Url;
use uuid::Uuid;

use crate::{IntoInsert, IntoSelect};

impl IntoSelect for FeedFindParams {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .column(Asterisk)
            .from(feeds::Entity)
            .apply_if(self.id, |query, id| {
                query.and_where(Expr::col((feeds::Entity, feeds::Column::Id)).eq(id.to_string()));
            })
            .apply_if(self.cursor, |query, cursor| {
                query.and_where(
                    Expr::col((feeds::Entity, feeds::Column::Link))
                        .gt(Expr::val(cursor.link.to_string())),
                );
            })
            .order_by((feeds::Entity, feeds::Column::Link), Order::Asc)
            .to_owned();

        if let Some(limit) = self.limit {
            query.limit(limit as u64);
        }

        query
    }
}

pub struct FeedUpsert {
    pub id: Uuid,
    pub link: Url,
    pub xml_url: Option<Url>,
    pub title: String,
    pub description: Option<String>,
    pub refreshed_at: Option<DateTime<Utc>>,
}

impl IntoInsert for FeedUpsert {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(feeds::Entity)
            .columns([
                feeds::Column::Id,
                feeds::Column::Link,
                feeds::Column::XmlUrl,
                feeds::Column::Title,
                feeds::Column::Description,
                feeds::Column::RefreshedAt,
            ])
            .values_panic([
                self.id.to_string().into(),
                self.link.to_string().into(),
                self.xml_url.map(String::from).into(),
                self.title.into(),
                self.description.into(),
                self.refreshed_at.map(|e| e.timestamp()).into(),
            ])
            .on_conflict(
                OnConflict::column(feeds::Column::Link)
                    .update_columns([
                        feeds::Column::XmlUrl,
                        feeds::Column::Title,
                        feeds::Column::Description,
                        feeds::Column::RefreshedAt,
                    ])
                    .to_owned(),
            )
            .returning_col(feeds::Column::Id)
            .to_owned()
    }
}

impl IntoSelect for FeedStreamUrlsParams {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .expr(Func::coalesce([
                Expr::col(feeds::Column::XmlUrl).into(),
                Expr::col(feeds::Column::Link).into(),
            ]))
            .from(feeds::Entity)
            .to_owned()
    }
}
