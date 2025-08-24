use std::collections::HashMap;

use bytes::Bytes;
use colette_core::common::RepositoryError;
use colette_opml::{Body, Opml, Outline, OutlineType};
use uuid::Uuid;

use crate::{Handler, SubscriptionQueryParams, SubscriptionQueryRepository};

#[derive(Debug, Clone)]
pub struct ExportSubscriptionsQuery {
    pub user_id: Uuid,
}

pub struct ExportSubscriptionsHandler<SQR: SubscriptionQueryRepository> {
    subscription_query_repository: SQR,
}

impl<SQR: SubscriptionQueryRepository> ExportSubscriptionsHandler<SQR> {
    pub fn new(subscription_query_repository: SQR) -> Self {
        Self {
            subscription_query_repository,
        }
    }
}

#[async_trait::async_trait]
impl<SQR: SubscriptionQueryRepository> Handler<ExportSubscriptionsQuery>
    for ExportSubscriptionsHandler<SQR>
{
    type Response = Bytes;
    type Error = ExportSubscriptionsError;

    async fn handle(&self, query: ExportSubscriptionsQuery) -> Result<Self::Response, Self::Error> {
        let mut outlines = Vec::<Outline>::new();
        let mut outline_map = HashMap::<Uuid, Outline>::new();

        let subscriptions = self
            .subscription_query_repository
            .query(SubscriptionQueryParams {
                user_id: query.user_id,
                ..Default::default()
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
