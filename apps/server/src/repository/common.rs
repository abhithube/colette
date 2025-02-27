use chrono::{DateTime, ParseResult, Utc};
use colette_core::{
    feed::ProcessedFeedEntry,
    filter::{BooleanOp, DateOp, NumberOp, TextOp},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseTransaction, DbErr,
    EntityTrait, LinkDef, Linked, QueryFilter, RelationTrait, sea_query::OnConflict,
};
use url::Url;
use uuid::Uuid;

use super::entity;

pub(crate) async fn upsert_feed<C: ConnectionTrait>(
    conn: &C,
    link: Url,
    xml_url: Option<Url>,
) -> Result<i32, DbErr> {
    let feed = entity::feeds::ActiveModel {
        link: ActiveValue::Set(link.into()),
        xml_url: ActiveValue::Set(xml_url.map(Into::into)),
        ..Default::default()
    };

    let mut keys = entity::feeds::Entity::insert(feed)
        .on_conflict(
            OnConflict::columns([entity::feeds::Column::Link])
                .update_columns([entity::feeds::Column::XmlUrl])
                .to_owned(),
        )
        .exec_with_returning_keys(conn)
        .await?;

    Ok(keys.swap_remove(0))
}

pub(crate) async fn upsert_entries<C: ConnectionTrait>(
    conn: &C,
    entries: Vec<ProcessedFeedEntry>,
    feed_id: i32,
) -> Result<(), DbErr> {
    let models = entries
        .into_iter()
        .map(|e| entity::feed_entries::ActiveModel {
            link: ActiveValue::Set(e.link.into()),
            title: ActiveValue::Set(e.title),
            published_at: ActiveValue::Set(e.published.to_rfc3339()),
            description: ActiveValue::Set(e.description),
            thumbnail_url: ActiveValue::Set(e.thumbnail.map(Into::into)),
            author: ActiveValue::Set(e.author),
            feed_id: ActiveValue::Set(feed_id),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    entity::feed_entries::Entity::insert_many(models)
        .do_nothing()
        .on_conflict(
            OnConflict::columns([
                entity::feed_entries::Column::FeedId,
                entity::feed_entries::Column::Link,
            ])
            .update_columns([
                entity::feed_entries::Column::Title,
                entity::feed_entries::Column::PublishedAt,
                entity::feed_entries::Column::Description,
                entity::feed_entries::Column::Author,
                entity::feed_entries::Column::ThumbnailUrl,
            ])
            .to_owned(),
        )
        .exec(conn)
        .await?;

    Ok(())
}

pub struct FeedEntryToUserFeed;

impl Linked for FeedEntryToUserFeed {
    type FromEntity = entity::feed_entries::Entity;
    type ToEntity = entity::user_feeds::Entity;

    fn link(&self) -> Vec<LinkDef> {
        vec![
            entity::feeds::Relation::FeedEntries.def().rev(),
            entity::feeds::Relation::UserFeeds.def(),
        ]
    }
}

pub(crate) async fn insert_many_user_feed_entries(
    tx: &DatabaseTransaction,
    feed_id: i32,
) -> Result<(), DbErr> {
    let models = entity::feed_entries::Entity::find()
        .find_also_linked(FeedEntryToUserFeed)
        .filter(entity::feed_entries::Column::FeedId.eq(feed_id))
        .all(tx)
        .await
        .map(|e| {
            e.into_iter()
                .filter_map(|(fe, uf)| {
                    uf.map(|uf| entity::user_feed_entries::ActiveModel {
                        id: ActiveValue::Set(Uuid::new_v4().to_string()),
                        feed_entry_id: ActiveValue::Set(fe.id),
                        user_feed_id: ActiveValue::Set(uf.id),
                        user_id: ActiveValue::Set(uf.user_id),
                        ..Default::default()
                    })
                })
                .collect::<Vec<_>>()
        })?;

    entity::user_feed_entries::Entity::insert_many(models)
        .on_conflict(
            OnConflict::columns([
                entity::user_feed_entries::Column::UserFeedId,
                entity::user_feed_entries::Column::FeedEntryId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec(tx)
        .await?;

    Ok(())
}

pub(crate) async fn upsert_bookmark<C: ConnectionTrait>(
    conn: &C,
    link: Url,
    title: String,
    thumbnail_url: Option<Url>,
    published_at: Option<DateTime<Utc>>,
    author: Option<String>,
    user_id: Uuid,
) -> Result<Uuid, DbErr> {
    let bookmark = entity::bookmarks::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4().into()),
        link: ActiveValue::Set(link.into()),
        title: ActiveValue::Set(title),
        thumbnail_url: ActiveValue::Set(thumbnail_url.map(Into::into)),
        published_at: ActiveValue::Set(published_at.map(|e| e.to_rfc3339())),
        author: ActiveValue::Set(author),
        user_id: ActiveValue::Set(user_id.into()),
        ..Default::default()
    };

    let mut keys = entity::bookmarks::Entity::insert(bookmark)
        .on_conflict(
            OnConflict::columns([
                entity::bookmarks::Column::UserId,
                entity::bookmarks::Column::Link,
            ])
            .update_columns([
                entity::bookmarks::Column::Title,
                entity::bookmarks::Column::ThumbnailUrl,
                entity::bookmarks::Column::PublishedAt,
                entity::bookmarks::Column::Author,
            ])
            .to_owned(),
        )
        .exec_with_returning_keys(conn)
        .await?;

    Ok(keys.swap_remove(0).parse().unwrap())
}

pub(crate) async fn upsert_tag(
    tx: &DatabaseTransaction,
    title: String,
    user_id: Uuid,
) -> Result<Uuid, DbErr> {
    let tag = entity::tags::Entity::find()
        .filter(entity::tags::Column::UserId.eq(user_id.to_string()))
        .filter(entity::tags::Column::Title.eq(title.clone()))
        .one(tx)
        .await?;

    let tag_id = match tag {
        Some(tag) => tag.id.parse().unwrap(),
        _ => {
            let id = Uuid::new_v4();
            let tag = entity::tags::ActiveModel {
                id: ActiveValue::Set(id.into()),
                title: ActiveValue::Set(title),
                user_id: ActiveValue::Set(user_id.into()),
                ..Default::default()
            };
            tag.insert(tx).await?;

            id
        }
    };

    Ok(tag_id)
}

pub(crate) fn parse_date(value: &str) -> ParseResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value).map(|e| e.to_utc())
}

pub(crate) trait ToColumn {
    fn to_column(&self) -> String;
}

pub(crate) trait ToSql {
    fn to_sql(&self) -> String;
}

impl ToSql for (String, &TextOp) {
    fn to_sql(&self) -> String {
        let (column, op) = self;

        match op {
            TextOp::Equals(value) => format!("{} = '{}'", column, value),
            TextOp::Contains(value) => format!("{} ILIKE '%{}%'", column, value),
            TextOp::StartsWith(value) => format!("{} ILIKE '{}%'", column, value),
            TextOp::EndsWith(value) => format!("{} ILIKE '%{}'", column, value),
        }
    }
}

impl ToSql for (String, &NumberOp) {
    fn to_sql(&self) -> String {
        let (column, op) = self;

        match op {
            NumberOp::Equals(value) => format!("{} = {}", column, value),
            NumberOp::LessThan(value) => format!("{} < {}", column, value),
            NumberOp::GreaterThan(value) => format!("{} > {}", column, value),
            NumberOp::Between(value) => format!(
                "{} > {} AND {} < {}",
                column, value.start, column, value.end
            ),
        }
    }
}

impl ToSql for (String, &BooleanOp) {
    fn to_sql(&self) -> String {
        let (column, op) = self;

        match op {
            BooleanOp::Equals(value) => format!("{} = {}", column, value),
        }
    }
}

impl ToSql for (String, &DateOp) {
    fn to_sql(&self) -> String {
        let (column, op) = self;

        match op {
            DateOp::Before(value) => format!("{} < '{}'", column, value),
            DateOp::After(value) => format!("{} > '{}'", column, value),
            DateOp::Between(value) => format!(
                "{} > '{}' AND {} < '{}'",
                column, value.start, column, value.end
            ),
            DateOp::InLast(value) => format!(
                "round((extract(epoch FROM now()) - extract(epoch FROM {})) * 1000) < '{}'",
                column, value
            ),
        }
    }
}
