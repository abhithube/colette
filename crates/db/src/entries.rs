use colette_core::{
    common::FindOneParams,
    entries::{EntriesFindManyParams, EntriesRepository, EntriesUpdateData, Error},
};
use colette_entities::{entry, profile_feed_entry, PfeWithEntry, ProfileFeedEntryToEntry};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, IntoActiveModel,
    QueryFilter, QueryOrder, QuerySelect, TransactionError, TransactionTrait,
};

use crate::PostgresRepository;

#[async_trait::async_trait]
impl EntriesRepository for PostgresRepository {
    async fn find_many_entries(
        &self,
        params: EntriesFindManyParams,
    ) -> Result<Vec<colette_core::Entry>, Error> {
        let mut conditions =
            Condition::all().add(profile_feed_entry::Column::ProfileId.eq(params.profile_id));
        if let Some(published_at) = params.published_at {
            conditions = conditions.add(entry::Column::PublishedAt.lt(published_at));
        }
        if let Some(feed_id) = params.feed_id {
            conditions = conditions.add(profile_feed_entry::Column::ProfileFeedId.eq(feed_id));
        }

        let models = profile_feed_entry::Entity::find()
            .find_also_linked(ProfileFeedEntryToEntry)
            .filter(conditions)
            .order_by_desc(entry::Column::PublishedAt)
            .order_by_asc(entry::Column::Title)
            .order_by_asc(profile_feed_entry::Column::Id)
            .limit(params.limit)
            .all(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let entries = models
            .into_iter()
            .filter_map(|(pfe, entry_opt)| {
                entry_opt.map(|entry| colette_core::Entry::from(PfeWithEntry { pfe, entry }))
            })
            .collect::<Vec<_>>();

        Ok(entries)
    }

    async fn find_one_entry(&self, params: FindOneParams) -> Result<colette_core::Entry, Error> {
        find_by_id(&self.db, params).await
    }

    async fn update_entry(
        &self,
        params: FindOneParams,
        data: EntriesUpdateData,
    ) -> Result<colette_core::Entry, Error> {
        self.db
            .transaction::<_, colette_core::Entry, Error>(|txn| {
                Box::pin(async move {
                    let Some(model) = profile_feed_entry::Entity::find_by_id(params.id)
                        .filter(profile_feed_entry::Column::ProfileId.eq(params.profile_id))
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
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

                    find_by_id(txn, params).await
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
    params: FindOneParams,
) -> Result<colette_core::Entry, Error> {
    let Some((pfe, Some(entry))) = profile_feed_entry::Entity::find_by_id(params.id)
        .find_also_linked(ProfileFeedEntryToEntry)
        .filter(profile_feed_entry::Column::ProfileId.eq(params.profile_id))
        .one(db)
        .await
        .map_err(|e| Error::Unknown(e.into()))?
    else {
        return Err(Error::NotFound(params.id));
    };

    let entry = colette_core::Entry::from(PfeWithEntry { pfe, entry });

    Ok(entry)
}
