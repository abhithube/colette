use chrono::{DateTime, Utc};
use colette_core::{
    common::{Paginated, PAGINATION_LIMIT},
    entries::{EntriesFindManyFilters, EntriesRepository, EntriesUpdateData, Error},
    Entry,
};
use colette_entities::{entry, profile_feed_entry, PfeWithEntry, ProfileFeedEntryToEntry};
use sea_orm::{
    sea_query::{Alias, Expr},
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, IntoActiveModel,
    QueryFilter, QueryOrder, QuerySelect, TransactionError, TransactionTrait,
};
use uuid::Uuid;

use crate::{utils, SqlRepository};

#[async_trait::async_trait]
impl EntriesRepository for SqlRepository {
    async fn find_many_entries(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor_raw: Option<String>,
        filters: Option<EntriesFindManyFilters>,
    ) -> Result<Paginated<Entry>, Error> {
        let mut conditions =
            Condition::all().add(profile_feed_entry::Column::ProfileId.eq(profile_id));
        if let Some(filters) = filters {
            if let Some(feed_id) = filters.feed_id {
                conditions = conditions.add(profile_feed_entry::Column::ProfileFeedId.eq(feed_id));
            }
        }
        if let Some(raw) = cursor_raw.as_deref() {
            let cursor =
                utils::decode_cursor::<Cursor>(raw).map_err(|e| Error::Unknown(e.into()))?;

            conditions = conditions.add(
                Expr::tuple([
                    Expr::col((Alias::new("r1"), entry::Column::PublishedAt)).into(),
                    Expr::col((profile_feed_entry::Entity, profile_feed_entry::Column::Id)).into(),
                ])
                .lte(Expr::tuple([
                    Expr::value(cursor.published_at),
                    Expr::value(cursor.id),
                ])),
            );
        }

        let models = profile_feed_entry::Entity::find()
            .find_also_linked(ProfileFeedEntryToEntry)
            .filter(conditions)
            .order_by_desc(Expr::col((Alias::new("r1"), entry::Column::PublishedAt)))
            .order_by_desc(profile_feed_entry::Column::Id)
            .limit(limit)
            .all(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let mut entries = models
            .into_iter()
            .filter_map(|(pfe, entry_opt)| {
                entry_opt.map(|entry| Entry::from(PfeWithEntry { pfe, entry }))
            })
            .collect::<Vec<_>>();
        let mut cursor: Option<String> = None;

        if entries.len() > PAGINATION_LIMIT {
            entries = entries.into_iter().take(PAGINATION_LIMIT).collect();

            if let Some(last) = entries.last() {
                let c = Cursor {
                    id: last.id,
                    published_at: last.published_at,
                };
                let encoded = utils::encode_cursor(&c).map_err(|e| Error::Unknown(e.into()))?;

                cursor = Some(encoded);
            }
        }

        Ok(Paginated::<Entry> {
            cursor,
            data: entries,
        })
    }

    async fn find_one_entry(&self, id: Uuid, profile_id: Uuid) -> Result<Entry, Error> {
        find_by_id(&self.db, id, profile_id).await
    }

    async fn update_entry(
        &self,
        id: Uuid,
        profile_id: Uuid,
        data: EntriesUpdateData,
    ) -> Result<Entry, Error> {
        self.db
            .transaction::<_, Entry, Error>(|txn| {
                Box::pin(async move {
                    let Some(model) = profile_feed_entry::Entity::find_by_id(id)
                        .filter(profile_feed_entry::Column::ProfileId.eq(profile_id))
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(id));
                    };
                    let mut active_model = model.into_active_model();

                    if let Some(has_read) = data.has_read {
                        active_model.has_read.set_if_not_equals(has_read);
                    }

                    if active_model.is_changed() {
                        active_model
                            .update(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    find_by_id(txn, id, profile_id).await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }
}

async fn find_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
) -> Result<Entry, Error> {
    let Some((pfe, Some(entry))) = profile_feed_entry::Entity::find_by_id(id)
        .find_also_linked(ProfileFeedEntryToEntry)
        .filter(profile_feed_entry::Column::ProfileId.eq(profile_id))
        .one(db)
        .await
        .map_err(|e| Error::Unknown(e.into()))?
    else {
        return Err(Error::NotFound(id));
    };

    let entry = Entry::from(PfeWithEntry { pfe, entry });

    Ok(entry)
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
struct Cursor {
    pub id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<DateTime<Utc>>,
}
