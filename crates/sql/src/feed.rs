use std::collections::HashMap;

use anyhow::anyhow;
use colette_core::{
    common::Paginated,
    feed::{
        Error, FeedCreateData, FeedFindManyFilters, FeedRepository, FeedUpdateData, StreamFeed,
    },
    Feed,
};
use colette_entities::PfWithFeedAndTagsAndUnreadCount;
use colette_utils::base_64;
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, DbErr, IntoActiveModel,
    TransactionError, TransactionTrait,
};
use uuid::Uuid;

use crate::queries;

pub struct FeedSqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl FeedSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl FeedRepository for FeedSqlRepository {
    async fn find_many(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor_raw: Option<String>,
        filters: Option<FeedFindManyFilters>,
    ) -> Result<Paginated<Feed>, Error> {
        find(&self.db, None, profile_id, limit, cursor_raw, filters).await
    }

    async fn find_one(&self, id: Uuid, profile_id: Uuid) -> Result<Feed, Error> {
        find_by_id(&self.db, id, profile_id).await
    }

    async fn create(&self, data: FeedCreateData) -> Result<Feed, Error> {
        self.db
            .transaction::<_, Feed, Error>(|txn| {
                Box::pin(async move {
                    let link = data.feed.link.to_string();
                    let result = queries::feed::insert(
                        txn,
                        link.clone(),
                        data.feed.title,
                        if data.url == link {
                            None
                        } else {
                            Some(data.url)
                        },
                    )
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;
                    let feed_id = result.last_insert_id;

                    let pf_id = match queries::profile_feed::insert(
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
                            let Some(model) = queries::profile_feed::select_by_unique_index(
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

                    let links = data
                        .feed
                        .entries
                        .iter()
                        .map(|e| e.link.to_string())
                        .collect::<Vec<_>>();

                    let insert_many = data
                        .feed
                        .entries
                        .into_iter()
                        .map(|e| queries::feed_entry::InsertMany {
                            link: e.link.to_string(),
                            title: e.title,
                            published_at: e.published.map(|e| e.into()),
                            description: e.description,
                            author: e.author,
                            thumbnail_url: e.thumbnail.map(String::from),
                        })
                        .collect::<Vec<_>>();

                    queries::feed_entry::insert_many(txn, insert_many, feed_id)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let fe_models = queries::feed_entry::select_many_in(txn, links)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let insert_many = fe_models
                        .into_iter()
                        .map(|e| queries::profile_feed_entry::InsertMany {
                            id: Uuid::new_v4(),
                            feed_entry_id: e.id,
                        })
                        .collect::<Vec<_>>();

                    queries::profile_feed_entry::insert_many(
                        txn,
                        insert_many,
                        pf_id,
                        data.profile_id,
                    )
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

                    find_by_id(txn, pf_id, data.profile_id).await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn update(
        &self,
        id: Uuid,
        profile_id: Uuid,
        data: FeedUpdateData,
    ) -> Result<Feed, Error> {
        self.db
            .transaction::<_, Feed, Error>(|txn| {
                Box::pin(async move {
                    let Some(pf_model) = queries::profile_feed::select_by_id(txn, id, profile_id)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(id));
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
                        queries::tag::insert_many(
                            txn,
                            tags.iter()
                                .map(|e| queries::tag::InsertMany {
                                    id: Uuid::new_v4(),
                                    title: e.to_owned(),
                                    profile_id,
                                })
                                .collect(),
                        )
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                        let tag_models = queries::tag::select_by_tags(txn, &tags)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                        let tag_ids = tag_models.iter().map(|e| e.id).collect::<Vec<_>>();

                        queries::profile_feed_tag::delete_many_not_in(txn, tag_ids.clone())
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        let insert_many = tag_ids
                            .iter()
                            .map(|e| queries::profile_feed_tag::InsertMany {
                                profile_feed_id: pf_model.id,
                                tag_id: *e,
                                profile_id,
                            })
                            .collect::<Vec<_>>();

                        queries::profile_feed_tag::insert_many(txn, insert_many)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    find_by_id(txn, id, profile_id).await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn delete(&self, id: Uuid, profile_id: Uuid) -> Result<(), Error> {
        let result = queries::profile_feed::delete_by_id(&self.db, id, profile_id)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(id));
        }

        Ok(())
    }

    async fn stream(&self) -> Result<BoxStream<Result<StreamFeed, Error>>, Error> {
        queries::feed::stream(&self.db)
            .await
            .map(|e| {
                e.map(|e| {
                    e.map(StreamFeed::from)
                        .map_err(|e| Error::Unknown(e.into()))
                })
                .map_err(|e| Error::Unknown(e.into()))
                .boxed()
            })
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn cleanup(&self) -> Result<(), Error> {
        self.db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    let result = queries::feed_entry::delete_many(txn).await?;
                    if result.rows_affected > 0 {
                        println!("Deleted {} orphaned feed entries", result.rows_affected);
                    }

                    let result = queries::feed::delete_many(txn).await?;
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

impl From<queries::feed::StreamSelect> for StreamFeed {
    fn from(value: queries::feed::StreamSelect) -> Self {
        Self {
            id: value.id,
            url: value.url,
        }
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
    let models = queries::profile_feed::select_with_feed(
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

    let tag_models = queries::profile_feed::load_tags(db, pf_models)
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

    let counts = queries::profile_feed_entry::count_many_in(db, pf_ids)
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

async fn find_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
) -> Result<Feed, Error> {
    let feeds = find(db, Some(id), profile_id, Some(1), None, None).await?;

    feeds.data.first().cloned().ok_or(Error::NotFound(id))
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub id: Uuid,
    pub title: String,
}
