use crate::queries;
use async_trait::async_trait;
use colette_core::{
    feeds::{Error, FeedCreateData, FeedsRepository},
    Feed,
};
use colette_database::{feed_entries, feeds, profile_feed_entries, profile_feeds, FindOneParams};
use sqlx::PgPool;

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

        let link = data.feed.link.as_str();
        let url = data.url.as_str();
        let x = feeds::InsertData {
            link,
            title: data.feed.title.as_str(),
            url: if url == link { None } else { Some(url) },
        };

        let feed_id = queries::feeds::insert(&mut *tx, x)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let profile_feed_id = queries::profile_feeds::create(
            &mut *tx,
            profile_feeds::InsertData {
                profile_id: data.profile_id.as_str(),
                feed_id,
            },
        )
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

        for e in data.feed.entries {
            let entry_id = queries::entries::insert(&mut *tx, (&e).into())
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            let feed_entry_id = queries::feed_entries::insert(
                &mut *tx,
                feed_entries::InsertData { feed_id, entry_id },
            )
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

            queries::profile_feed_entries::insert(
                &mut *tx,
                profile_feed_entries::InsertData {
                    profile_feed_id: profile_feed_id.as_str(),
                    feed_entry_id,
                },
            )
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        }

        let feed = queries::profile_feeds::find_one(
            &mut *tx,
            FindOneParams {
                id: profile_feed_id.as_str(),
                profile_id: data.profile_id.as_str(),
            },
        )
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(feed)
    }
}
