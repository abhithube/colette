use std::collections::HashMap;

use anyhow::anyhow;
use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Paginated, Updatable},
    feed::{
        Error, FeedCacheData, FeedCreateData, FeedFindManyFilters, FeedRepository, FeedUpdateData,
        ProcessedFeed,
    },
    Feed,
};
use colette_entity::PfWithFeedAndTagsAndUnreadCount;
use colette_utils::base_64;
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, DbErr, IntoActiveModel,
    TransactionError, TransactionTrait,
};
use uuid::Uuid;

use crate::query;

pub struct FeedSqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl FeedSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for FeedSqlRepository {
    type Params = IdParams;
    type Output = Result<Feed, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        find_by_id(&self.db, params).await
    }
}

#[async_trait::async_trait]
impl Creatable for FeedSqlRepository {
    type Data = FeedCreateData;
    type Output = Result<Feed, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        self.db
            .transaction::<_, Feed, Error>(|txn| {
                Box::pin(async move {
                    let feed_id = if let Some(feed) = data.feed {
                        create_feed_with_entries(txn, data.url, feed).await?
                    } else {
                        let Some(feed_model) = query::feed::select_by_url(txn, data.url.clone())
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                        else {
                            return Err(Error::Conflict(data.url));
                        };

                        feed_model.id
                    };

                    let pf_id = match query::profile_feed::insert(
                        txn,
                        Uuid::new_v4(),
                        data.profile_id,
                        feed_id,
                        data.folder_id.flatten(),
                    )
                    .await
                    {
                        Ok(result) => Ok(result.last_insert_id),
                        Err(DbErr::RecordNotInserted) => {
                            let Some(model) = query::profile_feed::select_by_unique_index(
                                txn,
                                data.profile_id,
                                feed_id,
                            )
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                            else {
                                return Err(Error::Unknown(anyhow!(
                                    "Failed to fetch created profile feed"
                                )));
                            };

                            Ok(model.id)
                        }
                        Err(e) => Err(Error::Unknown(e.into())),
                    }?;

                    let fe_models = query::feed_entry::select_many_by_feed_id(txn, feed_id)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let insert_many = fe_models
                        .into_iter()
                        .map(|e| query::profile_feed_entry::InsertMany {
                            id: Uuid::new_v4(),
                            feed_entry_id: e.id,
                        })
                        .collect::<Vec<_>>();

                    query::profile_feed_entry::insert_many(
                        txn,
                        insert_many,
                        pf_id,
                        data.profile_id,
                    )
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

                    find_by_id(txn, IdParams::new(pf_id, data.profile_id)).await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }
}

#[async_trait::async_trait]
impl Updatable for FeedSqlRepository {
    type Params = IdParams;

    type Data = FeedUpdateData;

    type Output = Result<Feed, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        self.db
            .transaction::<_, Feed, Error>(|txn| {
                Box::pin(async move {
                    let Some(pf_model) =
                        query::profile_feed::select_by_id(txn, params.id, params.profile_id)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };

                    let mut active_model = pf_model.clone().into_active_model();
                    if let Some(title) = data.title {
                        active_model.title.set_if_not_equals(title)
                    }
                    if let Some(folder_id) = data.folder_id {
                        active_model.folder_id.set_if_not_equals(folder_id)
                    }

                    if active_model.is_changed() {
                        active_model
                            .update(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    if let Some(tags) = data.tags {
                        query::tag::insert_many(
                            txn,
                            tags.iter()
                                .map(|e| query::tag::InsertMany {
                                    id: Uuid::new_v4(),
                                    title: e.to_owned(),
                                })
                                .collect(),
                            params.profile_id,
                        )
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                        let tag_models = query::tag::select_by_tags(txn, &tags)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                        let tag_ids = tag_models.iter().map(|e| e.id).collect::<Vec<_>>();

                        query::profile_feed_tag::delete_many_not_in(txn, tag_ids.clone())
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        let insert_many = tag_ids
                            .iter()
                            .map(|e| query::profile_feed_tag::InsertMany {
                                profile_feed_id: pf_model.id,
                                tag_id: *e,
                            })
                            .collect::<Vec<_>>();

                        query::profile_feed_tag::insert_many(txn, insert_many, params.profile_id)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    find_by_id(txn, params).await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }
}

#[async_trait::async_trait]
impl Deletable for FeedSqlRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let result = query::profile_feed::delete_by_id(&self.db, params.id, params.profile_id)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl FeedRepository for FeedSqlRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor_raw: Option<String>,
        filters: Option<FeedFindManyFilters>,
    ) -> Result<Paginated<Feed>, Error> {
        find(&self.db, None, profile_id, limit, cursor_raw, filters).await
    }

    async fn cache(&self, data: FeedCacheData) -> Result<(), Error> {
        self.db
            .transaction::<_, (), Error>(|txn| {
                Box::pin(async move {
                    create_feed_with_entries(txn, data.url, data.feed).await?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn stream(&self) -> Result<BoxStream<Result<(i32, String), Error>>, Error> {
        query::feed::stream(&self.db)
            .await
            .map(|e| {
                e.map(|e| e.map_err(|e| Error::Unknown(e.into())))
                    .map_err(|e| Error::Unknown(e.into()))
                    .boxed()
            })
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn cleanup(&self) -> Result<(), Error> {
        self.db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    let result = query::feed_entry::delete_many(txn).await?;
                    if result.rows_affected > 0 {
                        println!("Deleted {} orphaned feed entries", result.rows_affected);
                    }

                    let result = query::feed::delete_many(txn).await?;
                    if result.rows_affected > 0 {
                        println!("Deleted {} orphaned feeds", result.rows_affected);
                    }

                    Ok(())
                })
            })
            .await
            .map_err(|e| Error::Unknown(e.into()))
    }
}

async fn find<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor_raw: Option<String>,
    filters: Option<FeedFindManyFilters>,
) -> Result<Paginated<Feed>, Error> {
    let models = query::profile_feed::select_with_feed(
        db,
        id,
        profile_id,
        limit.map(|e| e + 1),
        cursor_raw.and_then(|e| base_64::decode::<Cursor>(&e).ok()),
        filters,
    )
    .await
    .map(|e| {
        e.into_iter()
            .filter_map(|(pf, feed_opt)| feed_opt.map(|feed| (pf, feed)))
            .collect::<Vec<_>>()
    })
    .map_err(|e| Error::Unknown(e.into()))?;
    let pf_models = models.iter().map(|e| e.0.to_owned()).collect::<Vec<_>>();
    let pf_ids = pf_models.iter().map(|e| e.id).collect::<Vec<_>>();

    let tag_models = query::profile_feed::load_tags(db, pf_models)
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

    let counts = query::profile_feed_entry::count_many_in(db, pf_ids)
        .await
        .map_err(|e| Error::Unknown(e.into()))?;
    let count_map: HashMap<Uuid, i64> = counts.into_iter().collect();

    let mut feeds = models
        .into_iter()
        .zip(tag_models.into_iter())
        .map(|((pf, feed), tags)| {
            let unread_count = count_map.get(&pf.id).cloned().unwrap_or_default();
            Feed::from(PfWithFeedAndTagsAndUnreadCount {
                pf,
                feed,
                tags,
                unread_count,
            })
        })
        .collect::<Vec<_>>();
    let mut cursor: Option<String> = None;

    if let Some(limit) = limit {
        let limit = limit as usize;
        if feeds.len() > limit {
            feeds = feeds.into_iter().take(limit).collect();

            if let Some(last) = feeds.last() {
                let c = Cursor {
                    id: last.id,
                    title: last
                        .title
                        .to_owned()
                        .unwrap_or(last.original_title.to_owned()),
                };
                let encoded = base_64::encode(&c)?;

                cursor = Some(encoded);
            }
        }
    }

    Ok(Paginated::<Feed> {
        cursor,
        data: feeds,
    })
}

async fn find_by_id<Db: ConnectionTrait>(db: &Db, params: IdParams) -> Result<Feed, Error> {
    let feeds = find(db, Some(params.id), params.profile_id, None, None, None).await?;

    feeds
        .data
        .first()
        .cloned()
        .ok_or(Error::NotFound(params.id))
}

async fn create_feed_with_entries<Db: ConnectionTrait>(
    db: &Db,
    url: String,
    feed: ProcessedFeed,
) -> Result<i32, Error> {
    let link = feed.link.to_string();
    let result = query::feed::insert(
        db,
        link.clone(),
        feed.title,
        if url == link { None } else { Some(url) },
    )
    .await
    .map_err(|e| Error::Unknown(e.into()))?;
    let feed_id = result.last_insert_id;

    let insert_many = feed
        .entries
        .into_iter()
        .map(|e| query::feed_entry::InsertMany {
            link: e.link.to_string(),
            title: e.title,
            published_at: e.published.into(),
            description: e.description,
            author: e.author,
            thumbnail_url: e.thumbnail.map(String::from),
        })
        .collect::<Vec<_>>();

    query::feed_entry::insert_many(db, insert_many, feed_id)
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

    Ok(feed_id)
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub id: Uuid,
    pub title: String,
}
