use colette_core::{
    Feed,
    feed::{Error, FeedParams, FeedRepository},
};
use colette_query::{
    IntoInsert, IntoSelect,
    feed::{FeedBase, FeedInsert},
    feed_entry::{FeedEntryInsert, FeedEntryInsertBatch},
};
use deadpool_sqlite::Pool;
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder as _;

use super::{IdRow, PreparedClient as _, SqliteRow};

#[derive(Debug, Clone)]
pub struct SqliteFeedRepository {
    pool: Pool,
}

impl SqliteFeedRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl FeedRepository for SqliteFeedRepository {
    async fn query(&self, params: FeedParams) -> Result<Vec<Feed>, Error> {
        let client = self.pool.get().await?;

        let feeds = client
            .interact(move |conn| {
                let (sql, values) = params.into_select().build_rusqlite(SqliteQueryBuilder);
                conn.query_prepared::<Feed>(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(feeds)
    }

    async fn save(&self, data: &mut Feed) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let feed = data.to_owned();

        data.id = client
            .interact(move |conn| {
                let tx = conn.transaction()?;

                let feed_id = {
                    let feed = FeedInsert {
                        feeds: [FeedBase {
                            id: feed.id,
                            source_url: feed.source_url.as_str(),
                            link: feed.link.as_str(),
                            title: &feed.title,
                            description: feed.description.as_deref(),
                            refreshed_at: feed.refreshed_at,
                            is_custom: feed.is_custom,
                        }],
                        upsert: true,
                    };

                    let (sql, values) = feed.into_insert().build_rusqlite(SqliteQueryBuilder);
                    let row = tx.query_one_prepared::<IdRow>(&sql, &values)?;

                    row.id
                };

                if let Some(ref entries) = feed.entries {
                    let entries = entries.iter().map(|e| FeedEntryInsert {
                        id: e.id,
                        link: e.link.as_str(),
                        title: &e.title,
                        published_at: e.published_at,
                        description: e.description.as_deref(),
                        author: e.author.as_deref(),
                        thumbnail_url: e.thumbnail_url.as_ref().map(|e| e.as_str()),
                        feed_id,
                    });

                    let (sql, values) = FeedEntryInsertBatch(entries)
                        .into_insert()
                        .build_rusqlite(SqliteQueryBuilder);

                    tx.execute_prepared(&sql, &values)?;
                }

                tx.commit()?;

                Ok::<_, Error>(feed_id)
            })
            .await
            .unwrap()?;

        Ok(())
    }
}

impl From<SqliteRow<'_>> for Feed {
    fn from(SqliteRow(value): SqliteRow<'_>) -> Self {
        Self {
            id: value.get_unwrap("id"),
            source_url: value.get_unwrap::<_, String>("source_url").parse().unwrap(),
            link: value.get_unwrap::<_, String>("link").parse().unwrap(),
            title: value.get_unwrap("title"),
            description: value.get_unwrap("description"),
            refreshed_at: value.get_unwrap("refreshed_at"),
            is_custom: value.get_unwrap("is_custom"),
            entries: None,
        }
    }
}
