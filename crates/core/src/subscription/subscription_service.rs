use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use bytes::{Buf, Bytes};
use colette_opml::{Body, Opml, Outline, OutlineType};
use url::Url;
use uuid::Uuid;

use super::{
    Error, ImportSubscriptionsParams, Subscription, SubscriptionCursor, SubscriptionFindParams,
    SubscriptionRepository,
};
use crate::{
    pagination::{Paginated, paginate},
    subscription::{
        SubscriptionBatchItem, SubscriptionInsertParams, SubscriptionLinkTagParams,
        SubscriptionUpdateParams,
    },
};

pub struct SubscriptionService {
    subscription_repository: Arc<dyn SubscriptionRepository>,
}

impl SubscriptionService {
    pub fn new(subscription_repository: Arc<dyn SubscriptionRepository>) -> Self {
        Self {
            subscription_repository,
        }
    }

    pub async fn list_subscriptions(
        &self,
        query: SubscriptionListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<Subscription, SubscriptionCursor>, Error> {
        let subscriptions = self
            .subscription_repository
            .find(SubscriptionFindParams {
                user_id: Some(user_id),
                tags: query.tags,
                cursor: query.cursor.map(|e| (e.title, e.id)),
                limit: query.limit.map(|e| e + 1),
                with_unread_count: query.with_unread_count,
                with_tags: query.with_tags,
                ..Default::default()
            })
            .await?;

        if let Some(limit) = query.limit {
            Ok(paginate(subscriptions, limit))
        } else {
            Ok(Paginated {
                items: subscriptions,
                ..Default::default()
            })
        }
    }

    pub async fn get_subscription(
        &self,
        query: SubscriptionGetQuery,
        user_id: Uuid,
    ) -> Result<Subscription, Error> {
        let mut subscriptions = self
            .subscription_repository
            .find(SubscriptionFindParams {
                id: Some(query.id),
                with_unread_count: query.with_unread_count,
                with_tags: query.with_tags,
                ..Default::default()
            })
            .await?;
        if subscriptions.is_empty() {
            return Err(Error::NotFound(query.id));
        }

        let subscription = subscriptions.swap_remove(0);
        if subscription.user_id != user_id {
            return Err(Error::Forbidden(query.id));
        }

        Ok(subscription)
    }

    pub async fn create_subscription(
        &self,
        data: SubscriptionCreate,
        user_id: Uuid,
    ) -> Result<Subscription, Error> {
        let id = self
            .subscription_repository
            .insert(SubscriptionInsertParams {
                title: data.title,
                description: data.description,
                feed_id: data.feed_id,
                user_id,
            })
            .await?;

        self.get_subscription(
            SubscriptionGetQuery {
                id,
                with_unread_count: false,
                with_tags: false,
            },
            user_id,
        )
        .await
    }

    pub async fn update_subscription(
        &self,
        id: Uuid,
        data: SubscriptionUpdate,
        user_id: Uuid,
    ) -> Result<Subscription, Error> {
        let Some(subscription) = self.subscription_repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if subscription.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.subscription_repository
            .update(SubscriptionUpdateParams {
                id,
                title: data.title,
                description: data.description,
            })
            .await?;

        self.get_subscription(
            SubscriptionGetQuery {
                id,
                with_unread_count: false,
                with_tags: false,
            },
            user_id,
        )
        .await
    }

    pub async fn delete_subscription(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        let Some(subscription) = self.subscription_repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if subscription.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.subscription_repository.delete_by_id(id).await?;

        Ok(())
    }

    pub async fn link_subscription_tags(
        &self,
        id: Uuid,
        data: LinkSubscriptionTags,
        user_id: Uuid,
    ) -> Result<(), Error> {
        let Some(subscription) = self.subscription_repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if subscription.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.subscription_repository
            .link_tags(SubscriptionLinkTagParams {
                subscription_id: id,
                tag_ids: data.tag_ids,
            })
            .await?;

        Ok(())
    }

    pub async fn import_subscriptions(&self, raw: Bytes, user_id: Uuid) -> Result<(), Error> {
        let opml = colette_opml::from_reader(raw.reader())?;

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
                user_id,
            })
            .await?;

        Ok(())
    }

    pub async fn export_subscriptions(&self, user_id: Uuid) -> Result<Bytes, Error> {
        let mut outlines = Vec::<Outline>::new();
        let mut outline_map = HashMap::<Uuid, Outline>::new();

        let subscriptions = self
            .subscription_repository
            .find(SubscriptionFindParams {
                user_id: Some(user_id),
                with_tags: true,
                ..Default::default()
            })
            .await?;

        for subscription in subscriptions {
            let outline = Outline {
                r#type: Some(OutlineType::default()),
                text: subscription.title.clone(),
                xml_url: Some(subscription.feed.source_url.into()),
                title: Some(subscription.title),
                html_url: Some(subscription.feed.link.into()),
                ..Default::default()
            };

            if let Some(tags) = subscription.tags
                && !tags.is_empty()
            {
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

#[derive(Debug, Clone, Default)]
pub struct SubscriptionListQuery {
    pub tags: Option<Vec<Uuid>>,
    pub cursor: Option<SubscriptionCursor>,
    pub limit: Option<usize>,
    pub with_unread_count: bool,
    pub with_tags: bool,
}

#[derive(Debug, Clone)]
pub struct SubscriptionGetQuery {
    pub id: Uuid,
    pub with_unread_count: bool,
    pub with_tags: bool,
}

#[derive(Debug, Clone)]
pub struct SubscriptionCreate {
    pub title: String,
    pub description: Option<String>,
    pub feed_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionUpdate {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct LinkSubscriptionTags {
    pub tag_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ImportSubscriptionsJobData {
    pub user_id: Uuid,
}
