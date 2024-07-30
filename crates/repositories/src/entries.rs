use anyhow::anyhow;
use chrono::{DateTime, Utc};
use colette_core::{
    common::FindOneParams,
    entries::{EntriesFindManyParams, EntriesRepository, EntriesUpdateData, Error},
    Entry,
};
use colette_entities::{entry, feed_entry, profile_feed, profile_feed_entry, PfeWithEntry};
use sea_orm::{
    prelude::Expr, sea_query::Alias, ColumnTrait, DatabaseConnection, EntityTrait, JoinType,
    Linked, QueryFilter, QueryOrder, QuerySelect, QueryTrait, RelationDef, RelationTrait,
    SelectModel, SelectTwo, Selector, Set, TransactionError, TransactionTrait,
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
        let models = select(None, params.profile_id)
            .apply_if(params.published_at, |query, v| {
                query.filter(entry::Column::PublishedAt.lt(v))
            })
            .apply_if(params.feed_id, |query, v| {
                query.filter(profile_feed_entry::Column::ProfileFeedId.eq(v))
            })
            .apply_if(params.has_read, |query, v| {
                query.filter(profile_feed_entry::Column::HasRead.eq(v))
            })
            // .order_by_desc(entry::Column::PublishedAt)
            .order_by_desc(Expr::col((Alias::new("r1"), entry::Column::PublishedAt)))
            .order_by_asc(profile_feed_entry::Column::Id)
            .limit(params.limit as u64)
            .all(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let mut entries: Vec<Entry> = vec![];
        for (pfe_model, entry_model) in models {
            let Some(entry_model) = entry_model else {
                return Err(Error::Unknown(anyhow!("Failed to fetch entries")));
            };

            entries.push(PfeWithEntry(pfe_model, entry_model).into());
        }

        Ok(entries)
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

#[derive(Debug)]
pub struct ProfileFeedEntryToEntry;

impl Linked for ProfileFeedEntryToEntry {
    type FromEntity = profile_feed_entry::Entity;
    type ToEntity = entry::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            profile_feed_entry::Relation::FeedEntry.def(),
            feed_entry::Relation::Entry.def(),
        ]
    }
}

fn select(
    id: Option<Uuid>,
    profile_id: Uuid,
) -> SelectTwo<profile_feed_entry::Entity, entry::Entity> {
    let query = match id {
        Some(id) => profile_feed_entry::Entity::find_by_id(id),
        None => profile_feed_entry::Entity::find(),
    };

    query
        .find_also_linked(ProfileFeedEntryToEntry)
        .join(
            JoinType::Join,
            profile_feed_entry::Relation::ProfileFeed.def(),
        )
        .filter(profile_feed::Column::ProfileId.eq(profile_id))
}
