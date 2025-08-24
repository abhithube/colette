use std::collections::{HashMap, HashSet};

use bytes::{Buf, Bytes};
use chrono::{DateTime, Utc};
use colette_core::{
    auth::UserId,
    bookmark::{BookmarkBatchItem, BookmarkRepository, ImportBookmarksParams},
    common::RepositoryError,
};
use colette_netscape::Item;
use colette_queue::{Job, JobProducer};
use tokio::sync::Mutex;
use url::Url;

use crate::Handler;

#[derive(Debug, Clone)]
pub struct ImportBookmarksCommand {
    pub raw: Bytes,
    pub user_id: UserId,
}

pub struct ImportBookmarksHandler<BR: BookmarkRepository, JP: JobProducer> {
    bookmark_repository: BR,

    import_bookmarks_producer: Mutex<JP>,
}

impl<BR: BookmarkRepository, JP: JobProducer> ImportBookmarksHandler<BR, JP> {
    pub fn new(bookmark_repository: BR, import_bookmarks_producer: JP) -> Self {
        Self {
            bookmark_repository,

            import_bookmarks_producer: Mutex::new(import_bookmarks_producer),
        }
    }
}

#[async_trait::async_trait]
impl<BR: BookmarkRepository, JP: JobProducer> Handler<ImportBookmarksCommand>
    for ImportBookmarksHandler<BR, JP>
{
    type Response = ();
    type Error = ImportBookmarksError;

    async fn handle(&self, cmd: ImportBookmarksCommand) -> Result<Self::Response, Self::Error> {
        let netscape = colette_netscape::from_reader(cmd.raw.reader())?;

        let mut stack: Vec<(Option<String>, Item)> =
            netscape.items.into_iter().map(|e| (None, e)).collect();

        let mut tag_set = HashSet::<String>::new();
        let mut bookmark_map = HashMap::<Url, BookmarkBatchItem>::new();

        while let Some((parent_title, item)) = stack.pop() {
            if !item.item.is_empty() {
                for child in item.item {
                    stack.push((Some(item.title.clone()), child));
                }

                tag_set.insert(item.title);
            } else if let Some(link) = item.href {
                let link = link.parse::<Url>().unwrap();

                let bookmark =
                    bookmark_map
                        .entry(link.clone())
                        .or_insert_with(|| BookmarkBatchItem {
                            link,
                            title: item.title,
                            thumbnail_url: None,
                            published_at: None,
                            author: None,
                            created_at: item
                                .add_date
                                .and_then(|e| DateTime::<Utc>::from_timestamp(e, 0)),
                            updated_at: item
                                .last_modified
                                .and_then(|e| DateTime::<Utc>::from_timestamp(e, 0)),
                            tag_titles: Vec::new(),
                        });

                if let Some(title) = parent_title {
                    bookmark.tag_titles.push(title);
                }
            }
        }

        self.bookmark_repository
            .import(ImportBookmarksParams {
                bookmark_items: bookmark_map.into_values().collect(),
                tag_titles: tag_set.into_iter().collect(),
                user_id: cmd.user_id,
            })
            .await?;

        let data = ImportBookmarksJobData {
            user_id: cmd.user_id,
        };
        let job = Job::create("import_bookmarks", data)?;

        let mut producer = self.import_bookmarks_producer.lock().await;

        producer.push(job).await?;

        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportBookmarksJobData {
    pub user_id: UserId,
}

#[derive(Debug, thiserror::Error)]
pub enum ImportBookmarksError {
    #[error(transparent)]
    Netscape(#[from] colette_netscape::Error),

    #[error(transparent)]
    Queue(#[from] colette_queue::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
