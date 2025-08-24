use std::collections::HashMap;

use bytes::Bytes;
use colette_common::RepositoryError;
use colette_netscape::{Item, Netscape};
use uuid::Uuid;

use crate::{BookmarkQueryParams, BookmarkQueryRepository, Handler};

#[derive(Debug, Clone)]
pub struct ExportBookmarksQuery {
    pub user_id: Uuid,
}

pub struct ExportBookmarksHandler<BQR: BookmarkQueryRepository> {
    bookmark_query_repository: BQR,
}

impl<BQR: BookmarkQueryRepository> ExportBookmarksHandler<BQR> {
    pub fn new(bookmark_query_repository: BQR) -> Self {
        Self {
            bookmark_query_repository,
        }
    }
}

#[async_trait::async_trait]
impl<BQR: BookmarkQueryRepository> Handler<ExportBookmarksQuery> for ExportBookmarksHandler<BQR> {
    type Response = Bytes;
    type Error = ExportBookmarksError;

    async fn handle(&self, query: ExportBookmarksQuery) -> Result<Self::Response, Self::Error> {
        let mut items = Vec::<Item>::new();
        let mut item_map = HashMap::<Uuid, Item>::new();

        let bookmarks = self
            .bookmark_query_repository
            .query(BookmarkQueryParams {
                user_id: query.user_id,
                ..Default::default()
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
