use colette_core::{
    backup::{BackupRepository, Error, ImportBookmarksParams, ImportFeedsParams},
    bookmark::{BookmarkUpsertParams, ProcessedBookmark},
};
use colette_query::{
    IntoInsert, IntoSelect, bookmark_tag::BookmarkTagUpsert, feed::FeedUpsert,
    subscription::SubscriptionUpsert, subscription_tag::SubscriptionTagUpsert, tag::TagUpsert,
};
use sea_orm::{ConnectionTrait, DatabaseConnection, TransactionTrait};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteBackupRepository {
    db: DatabaseConnection,
}

impl SqliteBackupRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl BackupRepository for SqliteBackupRepository {
    async fn import_feeds(&self, params: ImportFeedsParams) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let mut stack: Vec<(Option<Uuid>, colette_opml::Outline)> = params
            .outlines
            .into_iter()
            .map(|outline| (None, outline))
            .collect();

        while let Some((parent_id, outline)) = stack.pop() {
            if !outline.outline.is_empty() {
                let tag_id = {
                    let tag = TagUpsert {
                        id: Uuid::new_v4(),
                        title: outline.text,
                        user_id: params.user_id,
                    };

                    let result = tx
                        .query_one(tx.get_database_backend().build(&tag.clone().into_select()))
                        .await?;

                    match result {
                        Some(model) => model
                            .try_get_by_index::<String>(0)
                            .unwrap()
                            .parse()
                            .unwrap(),
                        _ => {
                            let id = Uuid::new_v4();

                            tx.execute(tx.get_database_backend().build(&tag.into_insert()))
                                .await?;

                            id
                        }
                    }
                };

                for child in outline.outline {
                    stack.push((Some(tag_id), child));
                }
            } else if let Some(link) = outline.html_url {
                let title = outline.title.unwrap_or(outline.text);

                let feed = FeedUpsert {
                    id: Uuid::new_v4(),
                    link: link.parse().unwrap(),
                    xml_url: outline.xml_url.map(|e| e.parse().unwrap()),
                    title: title.clone(),
                    description: None,
                    refreshed_at: None,
                };

                let feed_id = tx
                    .query_one(self.db.get_database_backend().build(&feed.into_insert()))
                    .await?
                    .unwrap()
                    .try_get_by_index::<String>(0)
                    .unwrap()
                    .parse::<Uuid>()
                    .unwrap();

                let subscription_id = {
                    let subscription = SubscriptionUpsert {
                        id: Uuid::new_v4(),
                        title,
                        feed_id,
                        user_id: params.user_id,
                    };

                    let result = tx
                        .query_one(
                            self.db
                                .get_database_backend()
                                .build(&subscription.into_insert()),
                        )
                        .await?
                        .unwrap();

                    result
                        .try_get_by_index::<String>(0)
                        .unwrap()
                        .parse::<Uuid>()
                        .unwrap()
                };

                if let Some(tag_id) = parent_id {
                    let subscription_tag = SubscriptionTagUpsert {
                        subscription_id,
                        tag_id,
                        user_id: params.user_id,
                    };

                    tx.execute(
                        self.db
                            .get_database_backend()
                            .build(&subscription_tag.into_insert()),
                    )
                    .await?;
                }
            }
        }

        tx.commit().await?;

        Ok(())
    }

    async fn import_bookmarks(&self, params: ImportBookmarksParams) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let mut stack: Vec<(Option<Uuid>, colette_netscape::Item)> =
            params.items.into_iter().map(|item| (None, item)).collect();

        while let Some((parent_id, item)) = stack.pop() {
            if !item.item.is_empty() {
                let tag = TagUpsert {
                    id: Uuid::new_v4(),
                    title: item.title,
                    user_id: params.user_id,
                };

                let result = tx
                    .query_one(tx.get_database_backend().build(&tag.clone().into_select()))
                    .await?;

                let tag_id = match result {
                    Some(model) => model
                        .try_get_by_index::<String>(0)
                        .unwrap()
                        .parse()
                        .unwrap(),
                    _ => {
                        let id = Uuid::new_v4();

                        tx.execute(tx.get_database_backend().build(&tag.into_insert()))
                            .await?;

                        id
                    }
                };

                for child in item.item {
                    stack.push((Some(tag_id), child));
                }
            } else if let Some(link) = item.href {
                let user_id = params.user_id;

                let params = BookmarkUpsertParams {
                    url: link.parse().unwrap(),
                    bookmark: ProcessedBookmark {
                        title: item.title,
                        ..Default::default()
                    },
                    user_id,
                };

                let bookmark_id: Uuid = tx
                    .query_one(self.db.get_database_backend().build(&params.into_insert()))
                    .await?
                    .unwrap()
                    .try_get_by_index::<String>(0)
                    .unwrap()
                    .parse()
                    .unwrap();

                if let Some(tag_id) = parent_id {
                    let bookmark_tag = BookmarkTagUpsert {
                        bookmark_id,
                        tag_id,
                        user_id,
                    };

                    tx.execute(
                        self.db
                            .get_database_backend()
                            .build(&bookmark_tag.into_insert()),
                    )
                    .await?;
                }
            }
        }

        tx.commit().await?;

        Ok(())
    }
}
