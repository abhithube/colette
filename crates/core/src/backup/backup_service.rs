use std::sync::Arc;

use bytes::{Buf, Bytes};
use chrono::{DateTime, Utc};
use colette_netscape::Netscape;
use colette_opml::{Body, Opml, OutlineType};
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;

use super::{Error, backup_repository::BackupRepository};
use crate::storage::DynStorage;

pub struct BackupService {
    backup_repository: Box<dyn BackupRepository>,
    import_feeds_storage: Arc<Mutex<DynStorage<ImportFeedsJob>>>,
    import_bookmarks_storage: Arc<Mutex<DynStorage<ImportBookmarksJob>>>,
}

impl BackupService {
    pub fn new(
        backup_repository: impl BackupRepository,
        import_feeds_storage: Arc<Mutex<DynStorage<ImportFeedsJob>>>,
        import_bookmarks_storage: Arc<Mutex<DynStorage<ImportBookmarksJob>>>,
    ) -> Self {
        Self {
            backup_repository: Box::new(backup_repository),
            import_feeds_storage,
            import_bookmarks_storage,
        }
    }

    pub async fn import_opml(&self, raw: Bytes, user_id: Uuid) -> Result<(), Error> {
        let opml = colette_opml::from_reader(raw.reader())?;

        let urls = opml
            .body
            .outlines
            .iter()
            .filter_map(|e| e.xml_url.as_deref().and_then(|e| Url::parse(e).ok()))
            .collect::<Vec<Url>>();

        self.backup_repository
            .import_feeds(opml.body.outlines, user_id)
            .await?;

        let mut storage = self.import_feeds_storage.lock().await;
        storage.push(ImportFeedsJob { urls }).await?;

        Ok(())
    }

    pub async fn export_opml(&self, user_id: Uuid) -> Result<Bytes, Error> {
        let input = self.backup_repository.export_outlines(user_id).await?;
        let outlines = build_outlines(&input, None);

        let opml = Opml {
            body: Body { outlines },
            ..Default::default()
        };

        let mut raw = Vec::<u8>::new();

        colette_opml::to_writer(&mut raw, opml)?;

        Ok(raw.into())
    }

    pub async fn import_netscape(&self, raw: Bytes, user_id: Uuid) -> Result<(), Error> {
        let netscape = colette_netscape::from_reader(raw.reader())?;

        let urls = netscape
            .items
            .iter()
            .filter_map(|e| e.href.as_deref().and_then(|e| Url::parse(e).ok()))
            .collect::<Vec<Url>>();

        self.backup_repository
            .import_bookmarks(netscape.items, user_id)
            .await?;

        let mut storage = self.import_bookmarks_storage.lock().await;
        storage.push(ImportBookmarksJob { urls, user_id }).await?;

        Ok(())
    }

    pub async fn export_netscape(&self, user_id: Uuid) -> Result<Bytes, Error> {
        let input = self.backup_repository.export_items(user_id).await?;
        let items = build_items(&input, None);

        let netscape = Netscape {
            items,
            ..Default::default()
        };

        let mut raw = Vec::<u8>::new();

        colette_netscape::to_writer(&mut raw, netscape)?;

        Ok(raw.into())
    }
}

fn build_outlines(input: &[Outline], parent_id: Option<Uuid>) -> Vec<colette_opml::Outline> {
    let mut output = Vec::new();

    for outline in input.iter().filter(|f| f.parent_id == parent_id) {
        if outline.xml_url.is_some() {
            let o = colette_opml::Outline {
                r#type: Some(OutlineType::default()),
                text: outline.text.clone(),
                title: Some(outline.text.clone()),
                xml_url: outline.xml_url.clone(),
                html_url: outline.html_url.clone(),
                ..Default::default()
            };
            output.push(o);
        } else {
            let children = build_outlines(input, Some(outline.id));
            if !children.is_empty() {
                let o = colette_opml::Outline {
                    text: outline.text.clone(),
                    outline: children,
                    ..Default::default()
                };
                output.push(o);
            }
        }
    }

    output
}

fn build_items(input: &[Item], parent_id: Option<Uuid>) -> Vec<colette_netscape::Item> {
    let mut output = Vec::new();

    for item in input.iter().filter(|f| f.parent_id == parent_id) {
        if item.href.is_some() {
            let i = colette_netscape::Item {
                title: item.title.clone(),
                add_date: item.add_date.map(|e| e.timestamp()),
                last_modified: item.last_modified.map(|e| e.timestamp()),
                href: item.href.clone(),
                ..Default::default()
            };
            output.push(i);
        } else {
            let children = build_items(input, Some(item.id));
            if !children.is_empty() {
                let o = colette_netscape::Item {
                    title: item.title.clone(),
                    add_date: item.add_date.map(|e| e.timestamp()),
                    last_modified: item.last_modified.map(|e| e.timestamp()),
                    item: children,
                    ..Default::default()
                };
                output.push(o);
            }
        }
    }

    output
}

#[derive(Debug, Clone, Default)]
pub struct Outline {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub text: String,
    pub xml_url: Option<String>,
    pub html_url: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Item {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub title: String,
    pub href: Option<String>,
    pub add_date: Option<DateTime<Utc>>,
    pub last_modified: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ImportFeedsJob {
    pub urls: Vec<Url>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ImportBookmarksJob {
    pub urls: Vec<Url>,
    pub user_id: Uuid,
}
