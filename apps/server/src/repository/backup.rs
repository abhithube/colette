use colette_core::backup::{BackupRepository, Error};
use colette_model::{bookmark_tags, subscription_tags, subscriptions};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, TransactionTrait,
    sea_query::{OnConflict, Query},
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
                    title.clone(),
                    None,
                    None,
                )
                .await?;

                let subscription_id: String = {
                    let query = Query::insert()
                        .into_table(subscriptions::Entity)
                        .columns([
                            subscriptions::Column::Id,
                            subscriptions::Column::Title,
                            subscriptions::Column::FeedId,
                            subscriptions::Column::UserId,
                        ])
                        .values_panic([
                            Uuid::new_v4().to_string().into(),
                            title.clone().into(),
                            feed_id.to_string().into(),
                            user_id.to_string().into(),
                        ])
                        .on_conflict(
                            OnConflict::columns([
                                subscriptions::Column::UserId,
                                subscriptions::Column::FeedId,
                            ])
                            .update_column(subscriptions::Column::Title)
                            .to_owned(),
                        )
                        .returning_col(subscriptions::Column::Id)
                        .to_owned();

                    let result = tx
                        .query_one(self.db.get_database_backend().build(&query))
                        .await?
                        .unwrap();

                    result.try_get_by_index::<String>(0).unwrap()
                };

                if let Some(tag_id) = parent_id {
                    let query = Query::insert()
                        .into_table(subscription_tags::Entity)
                        .columns([
                            subscription_tags::Column::SubscriptionId,
                            subscription_tags::Column::TagId,
                            subscription_tags::Column::UserId,
                        ])
                        .values_panic([
                            subscription_id.into(),
                            tag_id.to_string().into(),
                            user_id.to_string().into(),
                        ])
                        .on_conflict(
                            OnConflict::columns([
                                subscription_tags::Column::SubscriptionId,
                                subscription_tags::Column::TagId,
                            ])
                            .do_nothing()
                            .to_owned(),
                        )
                        .to_owned();

                    tx.execute(self.db.get_database_backend().build(&query))
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
                    let query = Query::insert()
                        .into_table(bookmark_tags::Entity)
                        .columns([
                            bookmark_tags::Column::BookmarkId,
                            bookmark_tags::Column::TagId,
                            bookmark_tags::Column::UserId,
                        ])
                        .values_panic([
                            bookmark_id.to_string().into(),
                            tag_id.to_string().into(),
                            user_id.to_string().into(),
                        ])
                        .on_conflict(
                            OnConflict::columns([
                                bookmark_tags::Column::BookmarkId,
                                bookmark_tags::Column::TagId,
                            ])
                            .do_nothing()
                            .to_owned(),
                        )
                        .to_owned();

                    tx.execute(self.db.get_database_backend().build(&query))
                        .await?;
                }
            }
        }

        tx.commit().await?;

        Ok(())
    }
}
