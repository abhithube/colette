use uuid::Uuid;

use super::{
    Error, LibraryItem,
    library_repository::{LibraryItemFindParams, LibraryRepository},
};
use crate::common::Paginated;

pub struct LibraryService {
    repository: Box<dyn LibraryRepository>,
}

impl LibraryService {
    pub fn new(repository: impl LibraryRepository) -> Self {
        Self {
            repository: Box::new(repository),
        }
    }

    pub async fn list_library_items(
        &self,
        query: LibraryItemListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<LibraryItem>, Error> {
        let library_items = self
            .repository
            .find(LibraryItemFindParams {
                folder_id: query.folder_id,
                user_id,
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: library_items,
            cursor: None,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct LibraryItemListQuery {
    pub folder_id: Option<Uuid>,
    pub cursor: Option<String>,
}
