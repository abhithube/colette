use colette_common::RepositoryError;
use colette_core::pagination::{Paginated, paginate};
use uuid::Uuid;

use crate::{Handler, PatCursor, PatQueryParams, PatQueryRepository, PersonalAccessTokenDto};

#[derive(Debug, Clone)]
pub struct ListPatsQuery {
    pub cursor: Option<PatCursor>,
    pub limit: Option<usize>,
    pub user_id: Uuid,
}

pub struct ListPatsHandler<PQR: PatQueryRepository> {
    pat_query_repository: PQR,
}

impl<PQR: PatQueryRepository> ListPatsHandler<PQR> {
    pub fn new(pat_query_repository: PQR) -> Self {
        Self {
            pat_query_repository,
        }
    }
}

#[async_trait::async_trait]
impl<PQR: PatQueryRepository> Handler<ListPatsQuery> for ListPatsHandler<PQR> {
    type Response = Paginated<PersonalAccessTokenDto, PatCursor>;
    type Error = ListPatsError;

    async fn handle(&self, query: ListPatsQuery) -> Result<Self::Response, Self::Error> {
        let pats = self
            .pat_query_repository
            .query(PatQueryParams {
                user_id: query.user_id,
                cursor: query.cursor.map(|e| e.created_at),
                limit: query.limit.map(|e| e + 1),
                ..Default::default()
            })
            .await?;

        if let Some(limit) = query.limit {
            Ok(paginate(pats, limit))
        } else {
            Ok(Paginated {
                items: pats,
                ..Default::default()
            })
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ListPatsError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
