use std::collections::HashMap;

use bytes::Bytes;
use colette_core::{
    auth::UserId,
    bookmark::{BookmarkFindParams, BookmarkRepository},
    common::RepositoryError,
};
use colette_netscape::{Item, Netscape};
use uuid::Uuid;

use crate::Handler;

#[derive(Debug, Clone)]
pub struct ExportBookmarksQuery {
    pub user_id: UserId,
}

pub struct ExportBookmarksHandler<BR: BookmarkRepository> {
    bookmark_repository: BR,
}

impl<BR: BookmarkRepository> ExportBookmarksHandler<BR> {
    pub fn new(bookmark_repository: BR) -> Self {
        Self {
            bookmark_repository,
        }
    }
}

#[async_trait::async_trait]
impl<BR: BookmarkRepository> Handler<ExportBookmarksQuery> for ExportBookmarksHandler<BR> {
    type Response = Bytes;
    type Error = ExportBookmarksError;

    async fn handle(&self, query: ExportBookmarksQuery) -> Result<Self::Response, Self::Error> {
        let mut items = Vec::<Item>::new();
        let mut item_map = HashMap::<Uuid, Item>::new();

        let bookmarks = self
            .bookmark_repository
            .find(BookmarkFindParams {
                user_id: query.user_id,
                id: None,
                filter: None,
                tags: None,
                cursor: None,
                limit: None,
            })
            .await?;

        for bookmark in bookmarks {
            let item = Item {
                title: bookmark.title,
                add_date: Some(bookmark.created_at.timestamp()),
                last_modified: Some(bookmark.updated_at.timestamp()),
                href: Some(bookmark.link.into()),
                ..Default::default()
            };

            if !bookmark.tags.is_empty() {
                for tag in bookmark.tags {
                    item_map
                        .entry(tag.id)
                        .or_insert_with(|| Item {
                            title: tag.title,
                            ..Default::default()
                        })
                        .item
                        .push(item.clone());
                }
            } else {
                items.push(item);
            }
        }

        items.append(&mut item_map.into_values().collect());

        let netscape = Netscape {
            items,
            ..Default::default()
        };

        let mut raw = Vec::<u8>::new();
        colette_netscape::to_writer(&mut raw, netscape)?;

        Ok(raw.into())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExportBookmarksError {
    #[error(transparent)]
    Netscape(#[from] colette_netscape::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
