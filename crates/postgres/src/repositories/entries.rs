use colette_core::{
    common::FindOneParams,
    entries::{EntriesFindManyParams, EntriesRepository, EntriesUpdateData, Error},
    Entry,
};
use colette_entities::{entries, feed_entries, profile_feed_entries, profile_feeds};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, JoinType, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, SelectModel, Selector, Set, SqlxPostgresConnector,
    TransactionTrait,
};
use sqlx::{
    types::chrono::{DateTime, Utc},
    PgPool,
};
use uuid::Uuid;

pub struct EntriesPostgresRepository {
    db: DatabaseConnection,
}

impl EntriesPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            db: SqlxPostgresConnector::from_sqlx_postgres_pool(pool),
        }
    }
}

#[async_trait::async_trait]
impl EntriesRepository for EntriesPostgresRepository {
    async fn find_many(&self, params: EntriesFindManyParams) -> Result<Vec<Entry>, Error> {
        let mut query = profile_feed_entries::Entity::find()
            .select_only()
            .columns(PROFILE_FEED_ENTRY_COLUMNS)
            .columns(ENTRY_COLUMNS)
            .column_as(profile_feed_entries::Column::ProfileFeedId, "feed_id")
            .join(
                JoinType::Join,
                profile_feed_entries::Relation::ProfileFeeds.def(),
            )
            .join(
                JoinType::Join,
                profile_feed_entries::Relation::FeedEntries.def(),
            )
            .join(JoinType::Join, feed_entries::Relation::Entries.def())
            .filter(profile_feeds::Column::ProfileId.eq(params.profile_id));

        if let Some(published_at) = params.published_at {
            query = query.filter(entries::Column::PublishedAt.lt(published_at))
        }
        if let Some(feed_id) = params.feed_id {
            query = query.filter(profile_feed_entries::Column::ProfileFeedId.eq(feed_id))
        }
        if let Some(has_read) = params.has_read {
            query = query.filter(profile_feed_entries::Column::HasRead.eq(has_read))
        }

        query
            .order_by_desc(entries::Column::PublishedAt)
            .order_by_asc(profile_feed_entries::Column::Id)
            .limit(params.limit as u64)
            .into_model::<EntrySelect>()
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Entry::from).collect::<Vec<_>>())
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn update(&self, params: FindOneParams, data: EntriesUpdateData) -> Result<Entry, Error> {
        self.db
            .transaction::<_, Entry, Error>(|txn| {
                Box::pin(async move {
                    let Some(mut entry) = entry_by_id(params.id, params.profile_id)
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };

                    let mut model = profile_feed_entries::ActiveModel {
                        id: Set(params.id),
                        ..Default::default()
                    };
                    if let Some(has_read) = data.has_read {
                        model.has_read = Set(has_read);
                    }

                    let model = profile_feed_entries::Entity::update(model)
                        .exec(txn)
                        .await
                        .map_err(|e| match e {
                            DbErr::RecordNotFound(_) => Error::NotFound(params.id),
                            _ => Error::Unknown(e.into()),
                        })?;

                    entry.has_read = model.has_read;

                    Ok(entry.into())
                })
            })
            .await
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
struct EntrySelect {
    id: Uuid,
    link: String,
    title: String,
    published_at: Option<DateTime<Utc>>,
    description: Option<String>,
    author: Option<String>,
    thumbnail_url: Option<String>,
    has_read: bool,
    feed_id: Uuid,
}

impl From<EntrySelect> for Entry {
    fn from(value: EntrySelect) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            published_at: value.published_at,
            description: value.description,
            author: value.author,
            thumbnail_url: value.thumbnail_url,
            has_read: value.has_read,
            feed_id: value.feed_id,
        }
    }
}

const PROFILE_FEED_ENTRY_COLUMNS: [profile_feed_entries::Column; 2] = [
    profile_feed_entries::Column::Id,
    profile_feed_entries::Column::HasRead,
];

const ENTRY_COLUMNS: [entries::Column; 5] = [
    entries::Column::Link,
    entries::Column::Title,
    entries::Column::PublishedAt,
    entries::Column::Author,
    entries::Column::ThumbnailUrl,
];

fn entry_by_id(id: Uuid, profile_id: Uuid) -> Selector<SelectModel<EntrySelect>> {
    profile_feed_entries::Entity::find_by_id(id)
        .select_only()
        .columns(PROFILE_FEED_ENTRY_COLUMNS)
        .columns(ENTRY_COLUMNS)
        .column_as(profile_feed_entries::Column::ProfileFeedId, "feed_id")
        .join(
            JoinType::Join,
            profile_feed_entries::Relation::ProfileFeeds.def(),
        )
        .join(
            JoinType::Join,
            profile_feed_entries::Relation::FeedEntries.def(),
        )
        .join(JoinType::Join, feed_entries::Relation::Entries.def())
        .filter(profile_feeds::Column::ProfileId.eq(profile_id))
        .into_model::<EntrySelect>()
}
