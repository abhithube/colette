use colette_core::{
    common::{Findable, IdParams, Updatable},
    feed_entry::{
        Cursor, Error, FeedEntryFindManyFilters, FeedEntryRepository, FeedEntryUpdateData,
    },
    FeedEntry,
};
use sqlx::{types::Uuid, PgExecutor, PgPool};

pub struct FeedEntrySqlRepository {
    pub(crate) pool: PgPool,
}

impl FeedEntrySqlRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for FeedEntrySqlRepository {
    type Params = IdParams;
    type Output = Result<FeedEntry, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        find_by_id(&self.pool, params).await
    }
}

#[async_trait::async_trait]
impl Updatable for FeedEntrySqlRepository {
    type Params = IdParams;
    type Data = FeedEntryUpdateData;
    type Output = Result<FeedEntry, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        colette_postgres::profile_feed_entry::update(
            &mut *tx,
            params.id,
            params.profile_id,
            data.has_read,
        )
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })?;

        let entry = find_by_id(&mut *tx, params).await?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(entry)
    }
}

#[async_trait::async_trait]
impl FeedEntryRepository for FeedEntrySqlRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
        filters: Option<FeedEntryFindManyFilters>,
    ) -> Result<Vec<FeedEntry>, Error> {
        find(&self.pool, None, profile_id, limit, cursor, filters).await
    }
}

async fn find(
    executor: impl PgExecutor<'_>,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<FeedEntryFindManyFilters>,
) -> Result<Vec<FeedEntry>, Error> {
    let mut feed_id: Option<Uuid> = None;
    let mut smart_feed_id: Option<Uuid> = None;
    let mut has_read: Option<bool> = None;
    let mut tags: Option<Vec<String>> = None;

    if let Some(filters) = filters {
        feed_id = filters.feed_id;
        smart_feed_id = filters.smart_feed_id;
        has_read = filters.has_read;
        tags = filters.tags;
    }

    colette_postgres::profile_feed_entry::select(
        executor,
        id,
        profile_id,
        feed_id,
        has_read,
        tags.as_deref(),
        smart_feed_id,
        cursor,
        limit,
    )
    .await
    .map_err(|e| Error::Unknown(e.into()))
}

async fn find_by_id(executor: impl PgExecutor<'_>, params: IdParams) -> Result<FeedEntry, Error> {
    let mut feed_entries = find(
        executor,
        Some(params.id),
        params.profile_id,
        None,
        None,
        None,
    )
    .await?;
    if feed_entries.is_empty() {
        return Err(Error::NotFound(params.id));
    }

    Ok(feed_entries.swap_remove(0))
}
