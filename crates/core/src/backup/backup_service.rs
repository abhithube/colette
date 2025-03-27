use std::collections::HashMap;

use bytes::{Buf, Bytes};
use colette_netscape::{Item, Netscape};
use colette_opml::{Body, Opml, Outline, OutlineType};
use colette_queue::JobProducer;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::{Error, ImportBookmarksData, ImportFeedsData, backup_repository::BackupRepository};
use crate::{
    bookmark::{BookmarkParams, BookmarkRepository},
    job::{Job, JobRepository},
    subscription::{SubscriptionParams, SubscriptionRepository},
};

pub struct BackupService {
    backup_repository: Box<dyn BackupRepository>,
    subscription_repository: Box<dyn SubscriptionRepository>,
    bookmark_repository: Box<dyn BookmarkRepository>,
    job_repository: Box<dyn JobRepository>,
    import_feeds_producer: Box<Mutex<dyn JobProducer>>,
    import_bookmarks_producer: Box<Mutex<dyn JobProducer>>,
}

impl BackupService {
    pub fn new(
        backup_repository: impl BackupRepository,
        subscription_repository: impl SubscriptionRepository,
        bookmark_repository: impl BookmarkRepository,
        job_repository: impl JobRepository,
        import_feeds_producer: impl JobProducer,
        import_bookmarks_producer: impl JobProducer,
    ) -> Self {
        Self {
            backup_repository: Box::new(backup_repository),
            subscription_repository: Box::new(subscription_repository),
            bookmark_repository: Box::new(bookmark_repository),
            job_repository: Box::new(job_repository),
            import_feeds_producer: Box::new(Mutex::new(import_feeds_producer)),
            import_bookmarks_producer: Box::new(Mutex::new(import_bookmarks_producer)),
        }
    }

    pub async fn import_opml(&self, raw: Bytes, user_id: String) -> Result<(), Error> {
        let opml = colette_opml::from_reader(raw.reader())?;

        self.backup_repository
            .import_feeds(ImportFeedsData {
                outlines: opml.body.outlines,
                user_id: user_id.clone(),
            })
            .await?;

        let data = serde_json::to_value(&ImportFeedsJobData { user_id })?;

        let job = Job::builder()
            .job_type("import_feeds".into())
            .data(data)
            .build();

        self.job_repository.save(&job).await?;

        let mut producer = self.import_feeds_producer.lock().await;

        producer.push(job.id).await?;

        Ok(())
    }

    pub async fn export_opml(&self, user_id: String) -> Result<Bytes, Error> {
        let mut outline_map = HashMap::<Uuid, Outline>::new();

        let subscriptions = self
            .subscription_repository
            .query(SubscriptionParams {
                user_id: Some(user_id),
                ..Default::default()
            })
            .await
            .unwrap();

        for subscription in subscriptions {
            let Some(feed) = subscription.feed else {
                continue;
            };

            let outline = Outline {
                r#type: Some(OutlineType::default()),
                text: subscription.title.clone(),
                xml_url: feed.xml_url.map(Into::into),
                title: Some(subscription.title),
                html_url: Some(feed.link.into()),
                ..Default::default()
            };

            if let Some(tags) = subscription.tags {
                for tag in tags {
                    outline_map
                        .entry(tag.id)
                        .or_insert_with(|| Outline {
                            text: tag.title,
                            ..Default::default()
                        })
                        .outline
                        .push(outline.clone());
                }
            }
        }

        let opml = Opml {
            body: Body {
                outlines: outline_map.into_values().collect(),
            },
            ..Default::default()
        };

        let mut raw = Vec::<u8>::new();

        colette_opml::to_writer(&mut raw, opml)?;

        Ok(raw.into())
    }

    pub async fn import_netscape(&self, raw: Bytes, user_id: String) -> Result<(), Error> {
        let netscape = colette_netscape::from_reader(raw.reader())?;

        self.backup_repository
            .import_bookmarks(ImportBookmarksData {
                items: netscape.items,
                user_id: user_id.clone(),
            })
            .await?;

        let data = serde_json::to_value(&ImportBookmarksJobData { user_id })?;

        let job = Job::builder()
            .job_type("import_bookmarks".into())
            .data(data)
            .build();

        self.job_repository.save(&job).await?;

        let mut producer = self.import_bookmarks_producer.lock().await;

        producer.push(job.id).await?;

        Ok(())
    }

    pub async fn export_netscape(&self, user_id: String) -> Result<Bytes, Error> {
        let mut item_map = HashMap::<Uuid, Item>::new();

        let bookmarks = self
            .bookmark_repository
            .query(BookmarkParams {
                user_id: Some(user_id),
                ..Default::default()
            })
            .await
            .unwrap();

        for bookmark in bookmarks {
            let item = Item {
                title: bookmark.title,
                add_date: Some(bookmark.created_at.timestamp()),
                last_modified: Some(bookmark.updated_at.timestamp()),
                href: Some(bookmark.link.into()),
                ..Default::default()
            };

            if let Some(tags) = bookmark.tags {
                for tag in tags {
                    item_map
                        .entry(tag.id)
                        .or_insert_with(|| Item {
                            title: tag.title,
                            ..Default::default()
                        })
                        .item
                        .push(item.clone());
                }
            }
        }

        let netscape = Netscape {
            items: item_map.into_values().collect(),
            ..Default::default()
        };

        let mut raw = Vec::<u8>::new();

        colette_netscape::to_writer(&mut raw, netscape)?;

        Ok(raw.into())
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ImportFeedsJobData {
    pub user_id: String,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ImportBookmarksJobData {
    pub user_id: String,
}
