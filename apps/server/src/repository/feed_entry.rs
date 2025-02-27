use chrono::{DateTime, Utc};
use colette_core::{
    FeedEntry,
    common::{Findable, IdParams, Updatable},
    feed_entry::{Error, FeedEntryFindParams, FeedEntryRepository, FeedEntryUpdateData},
    stream::{FeedEntryBooleanField, FeedEntryDateField, FeedEntryFilter, FeedEntryTextField},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter, QueryOrder, QuerySelect, QueryTrait, TransactionTrait, prelude::Expr,
    sea_query::Query,
};
use sqlx::types::Text;
use url::Url;
use uuid::Uuid;

use super::{
    common::{ToColumn, ToSql, parse_date},
    entity::{feed_entries, user_feed_entries, user_feed_tags},
};

#[derive(Debug, Clone)]
pub struct SqliteFeedEntryRepository {
    db: DatabaseConnection,
}

impl SqliteFeedEntryRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteFeedEntryRepository {
    type Params = FeedEntryFindParams;
    type Output = Result<Vec<FeedEntry>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let feed_entries = user_feed_entries::Entity::find()
            .find_also_related(feed_entries::Entity)
            .filter(user_feed_entries::Column::UserId.eq(params.user_id.to_string()))
            .apply_if(params.id, |query, id| {
                query.filter(user_feed_entries::Column::Id.eq(id.to_string()))
            })
            .apply_if(params.has_read, |query, has_read| {
                query.filter(user_feed_entries::Column::HasRead.eq(has_read))
            })
            .apply_if(params.tags, |query, tags| {
                query.filter(Expr::exists(
                    Query::select()
                        .expr(Expr::val(1))
                        .from(user_feed_tags::Entity)
                        .and_where(
                            Expr::col(user_feed_tags::Column::UserFeedId)
                                .eq(Expr::col(user_feed_entries::Column::UserFeedId)),
                        )
                        .and_where(
                            user_feed_tags::Column::TagId
                                .is_in(tags.into_iter().map(String::from).collect::<Vec<_>>()),
                        )
                        .to_owned(),
                ))
            })
            .apply_if(params.cursor, |query, cursor| {
                query.filter(
                    Expr::tuple([
                        Expr::col((feed_entries::Entity, feed_entries::Column::PublishedAt)).into(),
                        Expr::col((user_feed_entries::Entity, user_feed_entries::Column::Id))
                            .into(),
                    ])
                    .lt(Expr::tuple([
                        Expr::val(cursor.published_at.to_rfc3339()).into(),
                        Expr::val(cursor.id.to_string()).into(),
                    ])),
                )
            })
            .order_by_desc(feed_entries::Column::PublishedAt)
            .order_by_desc(user_feed_entries::Column::Id)
            .limit(params.limit.map(|e| e as u64))
            .all(&self.db)
            .await
            .map(|e| {
                e.into_iter()
                    .filter_map(|(ufe, fe)| fe.map(|fe| UfeWithFe { ufe, fe }.into()))
                    .collect()
            })?;

        Ok(feed_entries)
    }
}

#[async_trait::async_trait]
impl Updatable for SqliteFeedEntryRepository {
    type Params = IdParams;
    type Data = FeedEntryUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let tx = self.db.begin().await?;

        let Some(feed_entry) = user_feed_entries::Entity::find_by_id(params.id)
            .one(&tx)
            .await?
        else {
            return Err(Error::NotFound(params.id));
        };
        if feed_entry.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        let mut feed_entry = feed_entry.into_active_model();

        if let Some(has_read) = data.has_read {
            feed_entry.has_read = ActiveValue::Set(has_read.into());
        }

        if feed_entry.is_changed() {
            feed_entry.update(&tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }
}

impl FeedEntryRepository for SqliteFeedEntryRepository {}

struct UfeWithFe {
    ufe: user_feed_entries::Model,
    fe: feed_entries::Model,
}

impl From<UfeWithFe> for FeedEntry {
    fn from(value: UfeWithFe) -> Self {
        Self {
            id: value.ufe.id.parse().unwrap(),
            link: value.fe.link.parse().unwrap(),
            title: value.fe.title,
            published_at: parse_date(&value.fe.published_at).unwrap(),
            description: value.fe.description,
            author: value.fe.author,
            thumbnail_url: value.fe.thumbnail_url.and_then(|e| e.parse().ok()),
            has_read: value.ufe.has_read == 1,
            feed_id: value.ufe.user_feed_id.parse().unwrap(),
            created_at: parse_date(&value.ufe.created_at).ok(),
            updated_at: parse_date(&value.ufe.updated_at).ok(),
        }
    }
}

#[derive(sqlx::FromRow)]
pub(crate) struct FeedEntryRow {
    id: Uuid,
    link: Text<Url>,
    title: String,
    published_at: DateTime<Utc>,
    description: Option<String>,
    author: Option<String>,
    thumbnail_url: Option<Text<Url>>,
    has_read: bool,
    feed_id: Uuid,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
}

impl From<FeedEntryRow> for FeedEntry {
    fn from(value: FeedEntryRow) -> Self {
        Self {
            id: value.id,
            link: value.link.0,
            title: value.title,
            published_at: value.published_at,
            description: value.description,
            author: value.author,
            thumbnail_url: value.thumbnail_url.map(|e| e.0),
            has_read: value.has_read,
            feed_id: value.feed_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl ToColumn for FeedEntryTextField {
    fn to_column(&self) -> String {
        match self {
            Self::Link => "fe.link",
            Self::Title => "fe.title",
            Self::Description => "fe.description",
            Self::Author => "fe.author",
            Self::Tag => "t.title",
        }
        .into()
    }
}

impl ToColumn for FeedEntryBooleanField {
    fn to_column(&self) -> String {
        match self {
            Self::HasRead => "ufe.has_read",
        }
        .into()
    }
}

impl ToColumn for FeedEntryDateField {
    fn to_column(&self) -> String {
        match self {
            Self::PublishedAt => "fe.published_at",
            Self::CreatedAt => "ufe.created_at",
            Self::UpdatedAt => "ufe.updated_at",
        }
        .into()
    }
}

impl ToSql for FeedEntryFilter {
    fn to_sql(&self) -> String {
        match self {
            FeedEntryFilter::Text { field, op } => {
                let sql = (field.to_column(), op).to_sql();

                match field {
                    FeedEntryTextField::Tag => {
                        format!(
                            "EXISTS (SELECT 1 FROM user_feed_tags uft JOIN tags t ON t.id = uft.tag_id WHERE uft.user_feed_id = ufe.user_feed_id AND {})",
                            sql
                        )
                    }
                    _ => sql,
                }
            }
            FeedEntryFilter::Boolean { field, op } => (field.to_column(), op).to_sql(),
            FeedEntryFilter::Date { field, op } => (field.to_column(), op).to_sql(),
            FeedEntryFilter::And(filters) => {
                let conditions = filters.iter().map(|e| e.to_sql()).collect::<Vec<_>>();
                format!("({})", conditions.join(" AND "))
            }
            FeedEntryFilter::Or(filters) => {
                let conditions = filters.iter().map(|f| f.to_sql()).collect::<Vec<_>>();
                format!("({})", conditions.join(" OR "))
            }
            FeedEntryFilter::Not(filter) => {
                format!("NOT ({})", filter.to_sql())
            }
            _ => unreachable!(),
        }
    }
}
