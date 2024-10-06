use colette_core::backup::{BackupRepository, Error};
use colette_netscape::Item;
use colette_opml::Outline;
use futures::{future::BoxFuture, FutureExt};
use sea_orm::{prelude::Uuid, ConnectionTrait, DatabaseConnection, DbErr, TransactionTrait};

use crate::query;

pub struct BackupSqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl BackupSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

struct Parent {
    id: Uuid,
    title: String,
}

#[async_trait::async_trait]
impl BackupRepository for BackupSqlRepository {
    async fn import_opml(&self, outlines: Vec<Outline>, profile_id: Uuid) -> Result<(), Error> {
        fn recurse<Db: ConnectionTrait>(
            db: &Db,
            children: Vec<Outline>,
            parent_id: Option<Uuid>,
            profile_id: Uuid,
        ) -> BoxFuture<Result<(), DbErr>> {
            async move {
                for outline in children {
                    if let (Some(url), Some(link)) = (outline.xml_url, outline.html_url) {
                        let title = outline.title.unwrap_or(outline.text);

                        let inserted = query::feed::insert(db, link, title, Some(url)).await?;

                        let profile_feed_id = match query::profile_feed::select_by_unique_index(
                            db,
                            profile_id,
                            inserted.last_insert_id,
                        )
                        .await?
                        {
                            Some(model) => Ok(model.id),
                            None => match query::profile_feed::insert(
                                db,
                                Uuid::new_v4(),
                                None,
                                profile_id,
                                inserted.last_insert_id,
                            )
                            .await
                            {
                                Ok(model) => Ok(model.last_insert_id),
                                Err(e) => Err(e),
                            },
                        }?;

                        if let Some(tag_id) = parent_id {
                            query::profile_feed_tag::insert_many(
                                db,
                                vec![query::profile_feed_tag::InsertMany {
                                    profile_feed_id,
                                    tag_id,
                                }],
                                profile_id,
                            )
                            .await?;
                        }
                    } else if let Some(children) = outline.outline {
                        let model = match query::tag::select_by_title_and_parent(
                            db,
                            outline.text.clone(),
                            parent_id,
                            profile_id,
                        )
                        .await?
                        {
                            Some(model) => model,
                            None => {
                                query::tag::insert(
                                    db,
                                    Uuid::new_v4(),
                                    outline.text,
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
                    recurse(txn, outlines, None, profile_id).await?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn import_netscape(&self, items: Vec<Item>, profile_id: Uuid) -> Result<(), Error> {
        let mut tx = self
            .db
            .get_postgres_connection_pool()
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let mut stack: Vec<(Option<Parent>, Item)> =
            items.into_iter().map(|item| (None, item)).collect();

        while let Some((parent, mut item)) = stack.pop() {
            if item.item.is_some() {
                let title = if let Some(parent) = parent {
                    format!("{}/{}", parent.title, item.title)
                } else {
                    item.title
                };

                let tag_id = match colette_postgres::tag::select_by_title(
                    &mut *tx,
                    title.clone(),
                    profile_id,
                )
                .await
                {
                    Ok(id) => Ok(id),
                    _ => {
                        colette_postgres::tag::insert(
                            &mut *tx,
                            Uuid::new_v4(),
                            title.clone(),
                            profile_id,
                        )
                        .await
                    }
                }
                .map_err(|e| Error::Unknown(e.into()))?;

                if let Some(children) = item.item.take() {
                    for child in children.into_iter().rev() {
                        stack.push((
                            Some(Parent {
                                id: tag_id,
                                title: title.clone(),
                            }),
                            child,
                        ));
                    }
                }
            } else if let Some(link) = item.href {
                let bookmark_id = colette_postgres::bookmark::insert(
                    &mut *tx, link, item.title, None, None, None,
                )
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

                let profile_bookmark_id =
                    match colette_postgres::profile_bookmark::select_by_unique_index(
                        &mut *tx,
                        profile_id,
                        bookmark_id,
                    )
                    .await
                    {
                        Ok(id) => Ok(id),
                        _ => {
                            colette_postgres::profile_bookmark::insert(
                                &mut *tx,
                                Uuid::new_v4(),
                                bookmark_id,
                                profile_id,
                            )
                            .await
                        }
                    }
                    .map_err(|e| Error::Unknown(e.into()))?;

                if let Some(tag) = parent {
                    colette_postgres::profile_bookmark_tag::insert_many(
                        &mut *tx,
                        vec![colette_postgres::profile_bookmark_tag::InsertMany {
                            profile_bookmark_id,
                            tag_id: tag.id,
                        }],
                        profile_id,
                    )
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;
                }
            }
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }
}
