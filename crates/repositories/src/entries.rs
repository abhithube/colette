use chrono::{DateTime, Utc};
use colette_core::{
    common::FindOneParams,
    entries::{EntriesFindManyParams, EntriesRepository, EntriesUpdateData, Error},
    Entry,
};
use colette_entities::{entry, feed_entry, profile_feed, profile_feed_entry};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait, SelectModel, Selector, Set, TransactionError, TransactionTrait,
};
use uuid::Uuid;

pub struct EntriesSqlRepository {
    db: DatabaseConnection,
}

impl EntriesSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl EntriesRepository for EntriesSqlRepository {
    async fn find_many(&self, params: EntriesFindManyParams) -> Result<Vec<Entry>, Error> {
        let mut query = profile_feed_entry::Entity::find()
            .select_only()
            .columns(PROFILE_FEED_ENTRY_COLUMNS)
            .columns(ENTRY_COLUMNS)
            .column_as(profile_feed_entry::Column::ProfileFeedId, "feed_id")
            .join(
                JoinType::Join,
                profile_feed_entry::Relation::ProfileFeed.def(),
            )
            .join(
                JoinType::Join,
                profile_feed_entry::Relation::FeedEntry.def(),
            )
            .join(JoinType::Join, feed_entry::Relation::Entry.def())
            .filter(profile_feed::Column::ProfileId.eq(params.profile_id));

        if let Some(published_at) = params.published_at {
            query = query.filter(entry::Column::PublishedAt.lt(published_at))
        }
        if let Some(feed_id) = params.feed_id {
            query = query.filter(profile_feed_entry::Column::ProfileFeedId.eq(feed_id))
        }
        if let Some(has_read) = params.has_read {
            query = query.filter(profile_feed_entry::Column::HasRead.eq(has_read))
        }

        query
            .order_by_desc(entry::Column::PublishedAt)
            .order_by_asc(profile_feed_entry::Column::Id)
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

                    let mut model = profile_feed_entry::ActiveModel {
                        id: Set(params.id),
                        ..Default::default()
                    };
                    if let Some(has_read) = data.has_read {
                        model.has_read = Set(has_read);
                    }

                    let model = profile_feed_entry::Entity::update(model)
                        .exec(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    entry.has_read = model.has_read;

                    Ok(entry.into())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
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

const PROFILE_FEED_ENTRY_COLUMNS: [profile_feed_entry::Column; 2] = [
    profile_feed_entry::Column::Id,
    profile_feed_entry::Column::HasRead,
];

const ENTRY_COLUMNS: [entry::Column; 5] = [
    entry::Column::Link,
    entry::Column::Title,
    entry::Column::PublishedAt,
    entry::Column::Author,
    entry::Column::ThumbnailUrl,
];

fn entry_by_id(id: Uuid, profile_id: Uuid) -> Selector<SelectModel<EntrySelect>> {
    profile_feed_entry::Entity::find_by_id(id)
        .select_only()
        .columns(PROFILE_FEED_ENTRY_COLUMNS)
        .columns(ENTRY_COLUMNS)
        .column_as(profile_feed_entry::Column::ProfileFeedId, "feed_id")
        .join(
            JoinType::Join,
            profile_feed_entry::Relation::ProfileFeed.def(),
        )
        .join(
            JoinType::Join,
            profile_feed_entry::Relation::FeedEntry.def(),
        )
        .join(JoinType::Join, feed_entry::Relation::Entry.def())
        .filter(profile_feed::Column::ProfileId.eq(profile_id))
        .into_model::<EntrySelect>()
}
