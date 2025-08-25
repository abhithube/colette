use colette_common::RepositoryError;
use colette_crud::EntryFilter;
use uuid::Uuid;

use crate::{
    CollectionQueryRepository, EntryCursor, EntryDto, EntryQueryParams, EntryQueryRepository,
    Handler, Paginated, paginate,
};

#[derive(Debug, Clone)]
pub struct ListEntriesQuery {
    pub collection_id: Option<Uuid>,
    pub subscription_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<Uuid>>,
    pub cursor: Option<EntryCursor>,
    pub limit: Option<usize>,
    pub user_id: Uuid,
}

pub struct ListEntriesHandler<EQR: EntryQueryRepository, CQR: CollectionQueryRepository> {
    entry_query_repository: EQR,
    collection_query_repository: CQR,
}

impl<EQR: EntryQueryRepository, CQR: CollectionQueryRepository> ListEntriesHandler<EQR, CQR> {
    pub fn new(entry_query_repository: EQR, collection_query_repository: CQR) -> Self {
        Self {
            entry_query_repository,
            collection_query_repository,
        }
    }
}

impl<EQR: EntryQueryRepository, CQR: CollectionQueryRepository> Handler<ListEntriesQuery>
    for ListEntriesHandler<EQR, CQR>
{
    type Response = Paginated<EntryDto, EntryCursor>;
    type Error = ListEntriesError;

    async fn handle(&self, query: ListEntriesQuery) -> Result<Self::Response, Self::Error> {
        let filter = Option::<EntryFilter>::None;
        if let Some(collection_id) = query.collection_id {
            let Some(_) = self
                .collection_query_repository
                .query_by_id(collection_id, query.user_id)
                .await?
            else {
                return Ok(Paginated::default());
            };

            // filter = Some(collection.filter);
        }

        let entries = self
            .entry_query_repository
            .query(EntryQueryParams {
                user_id: query.user_id,
                subscription_id: query.subscription_id,
                has_read: query.has_read,
                tags: query.tags,
                filter,
                cursor: query.cursor.map(|e| (e.published_at, e.id)),
                limit: query.limit.map(|e| e + 1),
                ..Default::default()
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
