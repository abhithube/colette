use crate::{
    Handler,
    auth::UserId,
    collection::{CollectionFindParams, CollectionId, CollectionRepository},
    common::RepositoryError,
    entry::{EntryCursor, EntryDto, EntryFilter, EntryFindParams, EntryRepository},
    pagination::{Paginated, paginate},
    subscription::SubscriptionId,
    tag::TagId,
};

#[derive(Debug, Clone)]
pub struct ListEntriesQuery {
    pub collection_id: Option<CollectionId>,
    pub subscription_id: Option<SubscriptionId>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<TagId>>,
    pub cursor: Option<EntryCursor>,
    pub limit: Option<usize>,
    pub user_id: UserId,
}

pub struct ListEntriesHandler<ER: EntryRepository, CR: CollectionRepository> {
    entry_repository: ER,
    collection_repository: CR,
}

impl<ER: EntryRepository, CR: CollectionRepository> ListEntriesHandler<ER, CR> {
    pub fn new(entry_repository: ER, collection_repository: CR) -> Self {
        Self {
            entry_repository,
            collection_repository,
        }
    }
}

#[async_trait::async_trait]
impl<ER: EntryRepository, CR: CollectionRepository> Handler<ListEntriesQuery>
    for ListEntriesHandler<ER, CR>
{
    type Response = Paginated<EntryDto, EntryCursor>;
    type Error = ListEntriesError;

    async fn handle(&self, query: ListEntriesQuery) -> Result<Self::Response, Self::Error> {
        let filter = Option::<EntryFilter>::None;
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

        let entries = self
            .entry_repository
            .find(EntryFindParams {
                user_id: query.user_id,
                subscription_id: query.subscription_id,
                has_read: query.has_read,
                tags: query.tags,
                filter,
                cursor: query.cursor.map(|e| (e.published_at, e.id)),
                limit: query.limit.map(|e| e + 1),
                id: None,
            })
            .await?;

        if let Some(limit) = query.limit {
            Ok(paginate(entries, limit))
        } else {
            Ok(Paginated {
                items: entries,
                ..Default::default()
            })
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ListEntriesError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
