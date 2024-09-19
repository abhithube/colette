use colette_core::{
    backup::{BackupRepository, Error},
    common::{TagsLinkAction, TagsLinkData},
    tag::{TagFindManyFilters, TagType},
    Bookmark, Collection, Feed, Tag,
};
use colette_netscape::Item;
use colette_opml::Outline;
use futures::{future::BoxFuture, FutureExt};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, TransactionTrait};
use uuid::Uuid;

use crate::{bookmark, collection, feed, query, tag};

pub struct BackupSqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl BackupSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl BackupRepository for BackupSqlRepository {
    async fn import_opml(&self, outlines: Vec<Outline>, profile_id: Uuid) -> Result<(), Error> {
        fn recurse<Db: ConnectionTrait>(
            db: &Db,
            children: Vec<Outline>,
            tag: Option<String>,
            profile_id: Uuid,
        ) -> BoxFuture<Result<(), DbErr>> {
            async move {
                for outline in children {
                    if let (Some(url), Some(link)) = (outline.xml_url, outline.html_url) {
                        let title = outline.title.unwrap_or(outline.text);

                        let inserted = query::feed::insert(db, link, title, Some(url)).await?;

                        let profile_feed_id = match query::profile_feed::insert(
                            db,
                            Uuid::new_v4(),
                            None,
                            profile_id,
                            inserted.last_insert_id,
                        )
                        .await
                        {
                            Ok(model) => Ok(Some(model.last_insert_id)),
                            Err(DbErr::RecordNotInserted) => Ok(None),
                            Err(e) => Err(e),
                        }?;

                        if let Some(tag) = tag.clone() {
                            let profile_feed_id = match profile_feed_id {
                                Some(id) => id,
                                None => match query::profile_feed::select_by_unique_index(
                                    db,
                                    profile_id,
                                    inserted.last_insert_id,
                                )
                                .await?
                                {
                                    Some(model) => Ok(model.id),
                                    None => Err(DbErr::RecordNotFound(
                                        "Failed to fetch created profile feed".to_owned(),
                                    )),
                                }?,
                            };

                            feed::link_tags(
                                db,
                                profile_feed_id,
                                TagsLinkData {
                                    data: vec![tag],
                                    action: TagsLinkAction::Add,
                                },
                                profile_id,
                            )
                            .await?;
                        }
                    } else if let Some(children) = outline.outline {
                        recurse(db, children, Some(outline.text), profile_id).await?;
                    }
                }

                Ok(())
            }
            .boxed()
        }

        self.db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    recurse(txn, outlines, None, profile_id).await?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn export_opml(&self, profile_id: Uuid) -> Result<(Vec<Tag>, Vec<Feed>), Error> {
        self.db
            .transaction::<_, (Vec<Tag>, Vec<Feed>), Error>(|txn| {
                Box::pin(async move {
                    let tags = tag::find(
                        txn,
                        None,
                        profile_id,
                        None,
                        None,
                        Some(TagFindManyFilters {
                            tag_type: TagType::Feeds,
                        }),
                    )
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

                    let feeds = feed::find(txn, None, profile_id, None, None, None)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    Ok((tags, feeds))
                })
            })
            .await
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn import_netscape(&self, items: Vec<Item>, profile_id: Uuid) -> Result<(), Error> {
        fn recurse<Db: ConnectionTrait>(
            db: &Db,
            children: Vec<Item>,
            parent_id: Option<Uuid>,
            profile_id: Uuid,
        ) -> BoxFuture<Result<(), DbErr>> {
            async move {
                for item in children {
                    if let Some(link) = item.href {
                        let inserted =
                            query::bookmark::insert(db, link, item.title, None, None, None).await?;

                        let prev = query::profile_bookmark::select_last(db).await?;

                        let profile_bookmark_id = match query::profile_bookmark::insert(
                            db,
                            Uuid::new_v4(),
                            prev.map(|e| e.sort_index + 1).unwrap_or_default(),
                            profile_id,
                            inserted.last_insert_id,
                            parent_id,
                        )
                        .await
                        {
                            Ok(model) => Ok(Some(model.last_insert_id)),
                            Err(DbErr::RecordNotInserted) => Ok(None),
                            Err(e) => Err(e),
                        }?;

                        if let Some(tags) = item.tags {
                            let profile_bookmark_id = match profile_bookmark_id {
                                Some(id) => id,
                                None => match query::profile_bookmark::select_by_unique_index(
                                    db,
                                    profile_id,
                                    inserted.last_insert_id,
                                )
                                .await?
                                {
                                    Some(model) => Ok(model.id),
                                    None => Err(DbErr::RecordNotFound(
                                        "Failed to fetch created profile bookmark".to_owned(),
                                    )),
                                }?,
                            };

                            bookmark::link_tags(
                                db,
                                profile_bookmark_id,
                                TagsLinkData {
                                    data: tags,
                                    action: TagsLinkAction::Add,
                                },
                                profile_id,
                            )
                            .await?;
                        }
                    } else if let Some(children) = item.item {
                        let model = match query::collection::select_by_title_and_parent(
                            db,
                            item.title.clone(),
                            parent_id,
                            profile_id,
                        )
                        .await?
                        {
                            Some(model) => model,
                            None => {
                                query::collection::insert(
                                    db,
                                    Uuid::new_v4(),
                                    item.title,
                                    parent_id,
                                    profile_id,
                                )
                                .await?
                            }
                        };

                        recurse(db, children, Some(model.id), profile_id).await?;
                    }
                }

                Ok(())
            }
            .boxed()
        }

        self.db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    recurse(txn, items, None, profile_id).await?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn export_netscape(
        &self,
        profile_id: Uuid,
    ) -> Result<(Vec<Collection>, Vec<Bookmark>), Error> {
        self.db
            .transaction::<_, (Vec<Collection>, Vec<Bookmark>), Error>(|txn| {
                Box::pin(async move {
                    let collections = collection::find(txn, None, profile_id, None, None)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let bookmarks = bookmark::find(txn, None, profile_id, None, None, None)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    Ok((collections, bookmarks))
                })
            })
            .await
            .map_err(|e| Error::Unknown(e.into()))
    }
}
