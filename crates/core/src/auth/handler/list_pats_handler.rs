use crate::{
    Handler,
    auth::{PatCursor, PatFindParams, PatRepository, PersonalAccessToken, UserId},
    common::RepositoryError,
    pagination::{Paginated, paginate},
};

#[derive(Debug, Clone)]
pub struct ListPatsQuery {
    pub cursor: Option<PatCursor>,
    pub limit: Option<usize>,
    pub user_id: UserId,
}

pub struct ListPatsHandler {
    pat_repository: Box<dyn PatRepository>,
}

impl ListPatsHandler {
    pub fn new(pat_repository: impl PatRepository) -> Self {
        Self {
            pat_repository: Box::new(pat_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<ListPatsQuery> for ListPatsHandler {
    type Response = Paginated<PersonalAccessToken, PatCursor>;
    type Error = ListPatsError;

    async fn handle(&self, query: ListPatsQuery) -> Result<Self::Response, Self::Error> {
        let pats = self
            .pat_repository
            .find(PatFindParams {
                user_id: query.user_id,
                cursor: query.cursor.map(|e| e.created_at),
                limit: query.limit.map(|e| e + 1),
                id: None,
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
