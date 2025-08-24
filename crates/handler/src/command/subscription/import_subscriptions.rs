use std::collections::{HashMap, HashSet};

use bytes::{Buf, Bytes};
use colette_core::{
    auth::UserId,
    common::RepositoryError,
    subscription::{ImportSubscriptionsParams, SubscriptionBatchItem, SubscriptionRepository},
};
use colette_opml::Outline;
use url::Url;

use crate::{DEFAULT_INTERVAL, Handler};

#[derive(Debug, Clone)]
pub struct ImportSubscriptionsCommand {
    pub raw: Bytes,
    pub user_id: UserId,
}

pub struct ImportSubscriptionsHandler<SR: SubscriptionRepository> {
    subscription_repository: SR,
}

impl<SR: SubscriptionRepository> ImportSubscriptionsHandler<SR> {
    pub fn new(subscription_repository: SR) -> Self {
        Self {
            subscription_repository,
        }
    }
}

#[async_trait::async_trait]
impl<SR: SubscriptionRepository> Handler<ImportSubscriptionsCommand>
    for ImportSubscriptionsHandler<SR>
{
    type Response = ();
    type Error = ImportSubscriptionsError;

    async fn handle(&self, cmd: ImportSubscriptionsCommand) -> Result<Self::Response, Self::Error> {
        let opml = colette_opml::from_reader(cmd.raw.reader())?;

        let mut stack: Vec<(Option<String>, Outline)> =
            opml.body.outlines.into_iter().map(|e| (None, e)).collect();

        let mut tag_set = HashSet::<String>::new();
        let mut subscription_map = HashMap::<Url, SubscriptionBatchItem>::new();

        while let Some((parent_title, outline)) = stack.pop() {
            if !outline.outline.is_empty() {
                for child in outline.outline {
                    stack.push((Some(outline.text.clone()), child));
                }

                tag_set.insert(outline.text);
            } else if let Some(xml_url) = outline.xml_url {
                let xml_url = xml_url.parse::<Url>().unwrap();

                let subscription = subscription_map.entry(xml_url.clone()).or_insert_with(|| {
                    SubscriptionBatchItem {
                        feed_url: xml_url.clone(),
                        feed_link: outline
                            .html_url
                            .and_then(|e| e.parse().ok())
                            .unwrap_or(xml_url),
                        feed_title: outline.title.unwrap_or(outline.text),
                        tag_titles: Vec::new(),
                    }
                });

                if let Some(title) = parent_title {
                    subscription.tag_titles.push(title);
                }
            }
        }

        self.subscription_repository
            .import(ImportSubscriptionsParams {
                subscription_items: subscription_map.into_values().collect(),
                tag_titles: tag_set.into_iter().collect(),
                feed_refresh_interval: DEFAULT_INTERVAL,
                user_id: cmd.user_id,
            })
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportSubscriptionsJobData {
    pub user_id: UserId,
}

#[derive(Debug, thiserror::Error)]
pub enum ImportSubscriptionsError {
    #[error(transparent)]
    Opml(#[from] colette_opml::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
