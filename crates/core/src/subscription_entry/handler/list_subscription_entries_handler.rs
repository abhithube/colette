use crate::{
    Handler,
    auth::UserId,
    collection::{CollectionFindParams, CollectionId, CollectionRepository},
    common::RepositoryError,
    pagination::{Paginated, paginate},
    subscription::SubscriptionId,
    subscription_entry::{
        SubscriptionEntry, SubscriptionEntryCursor, SubscriptionEntryFilter,
        SubscriptionEntryFindParams, SubscriptionEntryRepository,
    },
    tag::TagId,
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

pub struct ListSubscriptionEntriesHandler<
    SER: SubscriptionEntryRepository,
    CR: CollectionRepository,
> {
    subscription_entry_repository: SER,
    collection_repository: CR,
}

impl<SER: SubscriptionEntryRepository, CR: CollectionRepository>
    ListSubscriptionEntriesHandler<SER, CR>
{
    pub fn new(subscription_entry_repository: SER, collection_repository: CR) -> Self {
        Self {
            subscription_entry_repository,
            collection_repository,
        }
    }
}

#[async_trait::async_trait]
impl<SER: SubscriptionEntryRepository, CR: CollectionRepository>
    Handler<ListSubscriptionEntriesQuery> for ListSubscriptionEntriesHandler<SER, CR>
{
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
                    user_id: query.user_id,
                    id: Some(collection_id),
                    cursor: None,
                    limit: None,
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
