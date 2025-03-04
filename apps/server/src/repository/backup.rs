use colette_core::backup::{BackupRepository, Error};
use colette_model::{bookmark_tags, subscription_tags, subscriptions};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    TransactionTrait,
};
use uuid::Uuid;

use super::common;

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
    async fn import_feeds(
        &self,
        outlines: Vec<colette_opml::Outline>,
        user_id: Uuid,
    ) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let mut stack: Vec<(Option<Uuid>, colette_opml::Outline)> = outlines
            .into_iter()
            .map(|outline| (None, outline))
            .collect();

        while let Some((parent_id, outline)) = stack.pop() {
            let title = outline.title.unwrap_or(outline.text);

            if !outline.outline.is_empty() {
                let tag_id = common::upsert_tag(&tx, title, user_id).await?;

                for child in outline.outline {
                    stack.push((Some(tag_id), child));
                }
            } else if let Some(link) = outline.html_url {
                let feed_id = common::upsert_feed(
                    &tx,
                    link.parse().unwrap(),
                    outline.xml_url.map(|e| e.parse().unwrap()),
                )
                .await?;

                let subscription_id = {
                    let subscription = subscriptions::Entity::find()
                        .filter(subscriptions::Column::FeedId.eq(feed_id))
                        .filter(subscriptions::Column::UserId.eq(user_id.to_string()))
                        .one(&tx)
                        .await?;

                    match subscription {
                        Some(tag) => tag.id.parse().unwrap(),
                        _ => {
                            let id = Uuid::new_v4();
                            let subscription = subscriptions::ActiveModel {
                                id: ActiveValue::Set(id.into()),
                                title: ActiveValue::Set(title),
                                feed_id: ActiveValue::Set(feed_id),
                                user_id: ActiveValue::Set(user_id.into()),
                                ..Default::default()
                            };
                            subscription.insert(&tx).await?;

                            id
                        }
                    }
                };

                if let Some(tag_id) = parent_id {
                    let subscription_tag = subscription_tags::ActiveModel {
                        subscription_id: ActiveValue::Set(subscription_id.into()),
                        tag_id: ActiveValue::Set(tag_id.into()),
                        user_id: ActiveValue::Set(user_id.into()),
                        ..Default::default()
                    };
                    subscription_tags::Entity::insert(subscription_tag)
                        .on_conflict_do_nothing()
                        .exec(&tx)
                        .await?;
                }
            }
        }

        tx.commit().await?;

        Ok(())
    }

    async fn import_bookmarks(
        &self,
        items: Vec<colette_netscape::Item>,
        user_id: Uuid,
    ) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let mut stack: Vec<(Option<Uuid>, colette_netscape::Item)> =
            items.into_iter().map(|item| (None, item)).collect();

        while let Some((parent_id, item)) = stack.pop() {
            if !item.item.is_empty() {
                let tag_id = common::upsert_tag(&tx, item.title, user_id).await?;

                for child in item.item {
                    stack.push((Some(tag_id), child));
                }
            } else if let Some(link) = item.href {
                let bookmark_id = common::upsert_bookmark(
                    &tx,
                    link.parse().unwrap(),
                    item.title,
                    None,
                    None,
                    None,
                    user_id,
                )
                .await?;

                if let Some(tag_id) = parent_id {
                    let bookmark_tag = bookmark_tags::ActiveModel {
                        bookmark_id: ActiveValue::Set(bookmark_id.into()),
                        tag_id: ActiveValue::Set(tag_id.into()),
                        user_id: ActiveValue::Set(user_id.into()),
                        ..Default::default()
                    };

                    bookmark_tags::Entity::insert(bookmark_tag)
                        .on_conflict_do_nothing()
                        .exec(&tx)
                        .await?;
                }
            }
        }

        tx.commit().await?;

        Ok(())
    }
}
