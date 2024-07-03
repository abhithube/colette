use async_trait::async_trait;
use colette_core::{
    feeds::{Error, FeedCreateData, FeedsRepository},
    Feed,
};
use sqlx::PgPool;

use crate::queries::{
    entries, feed_entries, feeds, profile_feed_entries, profile_feeds, FindOneParams,
};

pub struct FeedsPostgresRepository {
    pool: PgPool,
}

impl FeedsPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FeedsRepository for FeedsPostgresRepository {
    async fn create(&self, data: FeedCreateData) -> Result<Feed, Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let profile_id = data.profile_id.clone();

        let link = data.feed.link.as_str().to_owned();
        let x = feeds::InsertData {
            link: link.clone(),
            title: data.feed.title,
            url: if data.url == link {
                None
            } else {
                Some(data.url)
            },
        };

        let feed_id = feeds::insert(&mut *tx, x)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let profile_feed_id = profile_feeds::create(
            &mut *tx,
            profile_feeds::InsertData {
                profile_id: data.profile_id.clone(),
                feed_id,
            },
        )
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

        for e in data.feed.entries {
            let entry_id = entries::insert(&mut *tx, e.into())
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            let feed_entry_id =
                feed_entries::insert(&mut *tx, feed_entries::InsertData { feed_id, entry_id })
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

            profile_feed_entries::insert(
                &mut *tx,
                profile_feed_entries::InsertData {
                    profile_feed_id: profile_feed_id.clone(),
                    feed_entry_id,
                },
            )
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        }

        let feed = profile_feeds::find_one(
            &mut *tx,
            FindOneParams {
                id: profile_feed_id,
                profile_id,
            },
        )
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(feed)
    }
}