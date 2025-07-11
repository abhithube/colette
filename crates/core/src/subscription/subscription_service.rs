use std::{collections::HashMap, sync::Arc};

use bytes::{Buf, Bytes};
use chrono::Utc;
use colette_opml::{Body, Opml, Outline, OutlineType};
use colette_queue::JobProducer;
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;

use super::{
    Error, ImportSubscriptionsData, Subscription, SubscriptionCursor, SubscriptionParams,
    SubscriptionRepository,
};
use crate::{
    Feed, SubscriptionEntry, Tag,
    common::{PAGINATION_LIMIT, Paginated, Paginator},
    job::{Job, JobRepository},
    subscription_entry::{SubscriptionEntryParams, SubscriptionEntryRepository},
    tag::TagRepository,
};

pub struct SubscriptionService {
    subscription_repository: Arc<dyn SubscriptionRepository>,
    tag_repository: Arc<dyn TagRepository>,
    subscription_entry_repository: Arc<dyn SubscriptionEntryRepository>,
    job_repository: Arc<dyn JobRepository>,
    import_subscriptions_producer: Box<Mutex<dyn JobProducer>>,
}

impl SubscriptionService {
    pub fn new(
        subscription_repository: Arc<dyn SubscriptionRepository>,
        tag_repository: Arc<dyn TagRepository>,
        subscription_entry_repository: Arc<dyn SubscriptionEntryRepository>,
        job_repository: Arc<dyn JobRepository>,
        import_subscriptions_producer: impl JobProducer,
    ) -> Self {
        Self {
            subscription_repository,
            tag_repository,
            subscription_entry_repository,
            job_repository,
            import_subscriptions_producer: Box::new(Mutex::new(import_subscriptions_producer)),
        }
    }

    pub async fn list_subscriptions(
        &self,
        query: SubscriptionListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<Subscription>, Error> {
        let cursor = query
            .cursor
            .map(|e| Paginator::decode_cursor::<SubscriptionCursor>(&e))
            .transpose()?;

        let subscriptions = self
            .subscription_repository
            .query(SubscriptionParams {
                tags: query.tags,
                user_id: Some(user_id),
                cursor: cursor.map(|e| (e.title, e.id)),
                limit: Some(PAGINATION_LIMIT + 1),
                with_feed: query.with_feed,
                with_unread_count: query.with_unread_count,
                with_tags: query.with_tags,
                ..Default::default()
            })
            .await?;

        let data = Paginator::paginate(subscriptions, PAGINATION_LIMIT)?;

        Ok(data)
    }

    pub async fn get_subscription(
        &self,
        query: SubscriptionGetQuery,
        user_id: Uuid,
    ) -> Result<Subscription, Error> {
        let mut subscriptions = self
            .subscription_repository
            .query(SubscriptionParams {
                id: Some(query.id),
                with_feed: query.with_feed,
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
        let subscription = Subscription::builder()
            .title(data.title)
            .maybe_description(data.description)
            .feed_id(data.feed_id)
            .user_id(user_id)
            .build();

        self.subscription_repository.save(&subscription).await?;

        Ok(subscription)
    }

    pub async fn update_subscription(
        &self,
        id: Uuid,
        data: SubscriptionUpdate,
        user_id: Uuid,
    ) -> Result<Subscription, Error> {
        let Some(mut subscription) = self.subscription_repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if subscription.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        if let Some(title) = data.title {
            subscription.title = title;
        }
        if let Some(description) = data.description {
            subscription.description = description;
        }

        self.subscription_repository.save(&subscription).await?;

        Ok(subscription)
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
        let Some(mut subscription) = self.subscription_repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if subscription.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        let tags = if data.tag_ids.is_empty() {
            Vec::new()
        } else {
            self.tag_repository
                .find_by_ids(data.tag_ids)
                .await?
                .into_iter()
                .filter(|e| e.user_id == user_id)
                .collect()
        };

        subscription.tags = Some(tags);

        self.subscription_repository.save(&subscription).await?;

        Ok(())
    }

    pub async fn get_subscription_entry(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<SubscriptionEntry, Error> {
        let mut subscription_entries = self
            .subscription_entry_repository
            .query(SubscriptionEntryParams {
                feed_entry_id: Some(id),
                ..Default::default()
            })
            .await?;
        if subscription_entries.is_empty() {
            return Err(Error::NotFound(id));
        }

        let subscription_entry = subscription_entries.swap_remove(0);
        if subscription_entry.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        Ok(subscription_entry)
    }

    pub async fn mark_subscription_entry_as_read(
        &self,
        subscription_id: Uuid,
        feed_entry_id: Uuid,
        user_id: Uuid,
    ) -> Result<SubscriptionEntry, Error> {
        let Some(mut subscription_entry) = self
            .subscription_entry_repository
            .find_by_id(subscription_id, feed_entry_id)
            .await?
        else {
            return Err(Error::NotFound(feed_entry_id));
        };
        if subscription_entry.user_id != user_id {
            return Err(Error::Forbidden(feed_entry_id));
        }

        subscription_entry.has_read = Some(true);
        subscription_entry.read_at = Some(Utc::now());

        self.subscription_entry_repository
            .save(&subscription_entry)
            .await?;

        Ok(subscription_entry)
    }

    pub async fn mark_subscription_entry_as_unread(
        &self,
        subscription_id: Uuid,
        feed_entry_id: Uuid,
        user_id: Uuid,
    ) -> Result<SubscriptionEntry, Error> {
        let Some(mut subscription_entry) = self
            .subscription_entry_repository
            .find_by_id(subscription_id, feed_entry_id)
            .await?
        else {
            return Err(Error::NotFound(feed_entry_id));
        };
        if subscription_entry.user_id != user_id {
            return Err(Error::Forbidden(feed_entry_id));
        }

        subscription_entry.has_read = Some(false);

        self.subscription_entry_repository
            .save(&subscription_entry)
            .await?;

        Ok(subscription_entry)
    }

    pub async fn import_subscriptions(&self, raw: Bytes, user_id: Uuid) -> Result<(), Error> {
        let opml = colette_opml::from_reader(raw.reader())?;

        let mut stack: Vec<(Option<String>, Outline)> =
            opml.body.outlines.into_iter().map(|e| (None, e)).collect();

        let mut tag_map = HashMap::<String, Tag>::new();
        let mut subscription_map = HashMap::<Url, Subscription>::new();

        while let Some((parent_title, outline)) = stack.pop() {
            if !outline.outline.is_empty() {
                let tag = Tag::builder().title(outline.text).user_id(user_id).build();

                for child in outline.outline {
                    stack.push((Some(tag.title.clone()), child));
                }

                tag_map.insert(tag.title.clone(), tag);
            } else if let Some(xml_url) = outline.xml_url {
                let xml_url = xml_url.parse::<Url>().unwrap();

                let subscription = subscription_map.entry(xml_url.clone()).or_insert_with(|| {
                    let feed = Feed::builder()
                        .source_url(xml_url.clone())
                        .link(
                            outline
                                .html_url
                                .and_then(|e| e.parse().ok())
                                .unwrap_or(xml_url),
                        )
                        .title(outline.title.unwrap_or(outline.text))
                        .build();

                    Subscription::builder()
                        .title(feed.title.clone())
                        .feed_id(feed.id)
                        .user_id(user_id)
                        .build()
                });

                if let Some(title) = parent_title
                    && let Some(tag) = tag_map.get(&title)
                {
                    subscription
                        .tags
                        .get_or_insert_default()
                        .push(tag.to_owned());
                }
            }
        }

        self.subscription_repository
            .import(ImportSubscriptionsData {
                subscriptions: subscription_map.into_values().collect(),
                tags: tag_map.into_values().collect(),
                user_id,
            })
            .await?;

        let data = serde_json::to_value(&ImportSubscriptionsJobData { user_id })?;

        let job = Job::builder()
            .job_type("import_subscriptions".into())
            .data(data)
            .build();

        self.job_repository.save(&job).await?;

        let mut producer = self.import_subscriptions_producer.lock().await;

        producer.push(job.id).await?;

        Ok(())
    }

    pub async fn export_subscriptions(&self, user_id: Uuid) -> Result<Bytes, Error> {
        let mut outlines = Vec::<Outline>::new();
        let mut outline_map = HashMap::<Uuid, Outline>::new();

        let subscriptions = self
            .subscription_repository
            .query(SubscriptionParams {
                user_id: Some(user_id),
                with_feed: true,
                with_tags: true,
                ..Default::default()
            })
            .await?;

        for subscription in subscriptions {
            let Some(feed) = subscription.feed else {
                continue;
            };

            let outline = Outline {
                r#type: Some(OutlineType::default()),
                text: subscription.title.clone(),
                xml_url: Some(feed.source_url.into()),
                title: Some(subscription.title),
                html_url: Some(feed.link.into()),
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
    pub cursor: Option<String>,
    pub with_feed: bool,
    pub with_unread_count: bool,
    pub with_tags: bool,
}

#[derive(Debug, Clone)]
pub struct SubscriptionGetQuery {
    pub id: Uuid,
    pub with_feed: bool,
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
