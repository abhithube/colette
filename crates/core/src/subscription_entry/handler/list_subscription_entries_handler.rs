use crate::{
    Handler, RepositoryError,
    collection::{CollectionFindParams, CollectionId, CollectionRepository},
    pagination::{Paginated, paginate},
    subscription::SubscriptionId,
    subscription_entry::{
        SubscriptionEntry, SubscriptionEntryCursor, SubscriptionEntryFilter,
        SubscriptionEntryFindParams, SubscriptionEntryRepository,
    },
    tag::TagId,
    user::UserId,
};

#[derive(Debug, Clone)]
pub struct ListSubscriptionEntriesQuery {
    pub collection_id: Option<CollectionId>,
    pub subscription_id: Option<SubscriptionId>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<TagId>>,
    pub cursor: Option<SubscriptionEntryCursor>,
    pub limit: Option<usize>,
    pub user_id: UserId,
}

pub struct ListSubscriptionEntriesHandler {
    subscription_entry_repository: Box<dyn SubscriptionEntryRepository>,
    collection_repository: Box<dyn CollectionRepository>,
}

impl ListSubscriptionEntriesHandler {
    pub fn new(
        subscription_entry_repository: impl SubscriptionEntryRepository,
        collection_repository: impl CollectionRepository,
    ) -> Self {
        Self {
            subscription_entry_repository: Box::new(subscription_entry_repository),
            collection_repository: Box::new(collection_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<ListSubscriptionEntriesQuery> for ListSubscriptionEntriesHandler {
    type Response = Paginated<SubscriptionEntry, SubscriptionEntryCursor>;
    type Error = ListSubscriptionEntriesError;

    async fn handle(
        &self,
        query: ListSubscriptionEntriesQuery,
    ) -> Result<Self::Response, Self::Error> {
        let filter = Option::<SubscriptionEntryFilter>::None;
        if let Some(collection_id) = query.collection_id {
            let collections = self
                .collection_repository
                .find(CollectionFindParams {
                    id: Some(collection_id),
                    user_id: Some(query.user_id),
                    ..Default::default()
                })
                .await?;
            if collections.is_empty() {
                return Ok(Paginated {
                    items: Default::default(),
                    cursor: None,
                });
            }

            // filter = Some(collections.swap_remove(0).filter);
        }

        let subscription_entries = self
            .subscription_entry_repository
            .find(SubscriptionEntryFindParams {
                filter,
                subscription_id: query.subscription_id,
                has_read: query.has_read,
                tags: query.tags,
                user_id: Some(query.user_id),
                cursor: query.cursor.map(|e| (e.published_at, e.id)),
                limit: query.limit.map(|e| e + 1),
                with_feed_entry: true,
                ..Default::default()
            })
            .await?;

        if let Some(limit) = query.limit {
            Ok(paginate(subscription_entries, limit))
        } else {
            Ok(Paginated {
                items: subscription_entries,
                ..Default::default()
            })
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ListSubscriptionEntriesError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
