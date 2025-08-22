use std::collections::HashMap;

use bytes::Bytes;
use colette_opml::{Body, Opml, Outline, OutlineType};
use uuid::Uuid;

use crate::{
    Handler,
    auth::UserId,
    common::RepositoryError,
    subscription::{SubscriptionFindParams, SubscriptionRepository},
};

#[derive(Debug, Clone)]
pub struct ExportSubscriptionsQuery {
    pub user_id: UserId,
}

pub struct ExportSubscriptionsHandler {
    subscription_repository: Box<dyn SubscriptionRepository>,
}

impl ExportSubscriptionsHandler {
    pub fn new(subscription_repository: impl SubscriptionRepository) -> Self {
        Self {
            subscription_repository: Box::new(subscription_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<ExportSubscriptionsQuery> for ExportSubscriptionsHandler {
    type Response = Bytes;
    type Error = ExportSubscriptionsError;

    async fn handle(&self, query: ExportSubscriptionsQuery) -> Result<Self::Response, Self::Error> {
        let mut outlines = Vec::<Outline>::new();
        let mut outline_map = HashMap::<Uuid, Outline>::new();

        let subscriptions = self
            .subscription_repository
            .find(SubscriptionFindParams {
                user_id: query.user_id,
                id: None,
                tags: None,
                cursor: None,
                limit: None,
            })
            .await?;

        for subscription in subscriptions {
            let outline = Outline {
                r#type: Some(OutlineType::default()),
                text: subscription.title.clone(),
                xml_url: Some(subscription.source_url.into()),
                title: Some(subscription.title),
                html_url: Some(subscription.link.into()),
                ..Default::default()
            };

            if !subscription.tags.is_empty() {
                for tag in subscription.tags {
                    outline_map
                        .entry(tag.id)
                        .or_insert_with(|| Outline {
                            text: tag.title,
                            ..Default::default()
                        })
                        .outline
                        .push(outline.clone());
                }
            } else {
                outlines.push(outline);
            }
        }

        outlines.append(&mut outline_map.into_values().collect());

        let opml = Opml {
            body: Body { outlines },
            ..Default::default()
        };

        let mut raw = Vec::<u8>::new();
        colette_opml::to_writer(&mut raw, opml)?;

        Ok(raw.into())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExportSubscriptionsError {
    #[error(transparent)]
    Opml(#[from] colette_opml::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
