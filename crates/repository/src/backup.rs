use colette_core::backup::{BackupRepository, Error};
use colette_netscape::Item;
use colette_opml::Outline;
use futures::{future::BoxFuture, FutureExt};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, TransactionTrait};
use uuid::Uuid;

use crate::query;

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
            tag_id: Option<Uuid>,
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
                            profile_id,
                            inserted.last_insert_id,
                        )
                        .await
                        {
                            Ok(model) => Ok(Some(model.last_insert_id)),
                            Err(DbErr::RecordNotInserted) => Ok(None),
                            Err(e) => Err(e),
                        }?;

                        if let Some(tag_id) = tag_id {
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
                        let model =
                            match query::tag::select_by_title(db, outline.text.clone(), profile_id)
                                .await?
                            {
                                Some(model) => model,
                                None => {
                                    query::tag::insert(db, Uuid::new_v4(), outline.text, profile_id)
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
        fn recurse<Db: ConnectionTrait>(
            db: &Db,
            children: Vec<Item>,
            parent: Option<Uuid>,
            profile_id: Uuid,
        ) -> BoxFuture<Result<(), DbErr>> {
            async move {
                let _create_folder = children.iter().any(|e| e.item.is_some());

                for item in children {
                    if let Some(link) = item.href {
                        let inserted =
                            query::bookmark::insert(db, link, item.title, None, None, None).await?;

                        let prev = query::profile_bookmark::select_last(db).await?;

                        match query::profile_bookmark::insert(
                            db,
                            Uuid::new_v4(),
                            prev.map(|e| e.sort_index + 1).unwrap_or_default(),
                            profile_id,
                            inserted.last_insert_id,
                            parent,
                        )
                        .await
                        {
                            Ok(_) | Err(DbErr::RecordNotInserted) => Ok(()),
                            Err(e) => Err(e),
                        }?
                    } else if let Some(children) = item.item {
                        // let id = if create_folder {
                        //     let model = match query::folder::select_by_title_and_parent(
                        //         db,
                        //         item.title.clone(),
                        //         parent,
                        //         profile_id,
                        //     )
                        //     .await?
                        //     {
                        //         Some(model) => model,
                        //         None => {
                        //             query::folder::insert(
                        //                 db,
                        //                 Uuid::new_v4(),
                        //                 item.title,
                        //                 parent,
                        //                 profile_id,
                        //             )
                        //             .await?
                        //         }
                        //     };

                        //     model.id
                        // } else {
                        //     let model = match query::collection::select_by_title_and_parent(
                        //         db,
                        //         item.title.clone(),
                        //         parent,
                        //         profile_id,
                        //     )
                        //     .await?
                        //     {
                        //         Some(model) => model,
                        //         None => {
                        //             query::collection::insert(
                        //                 db,
                        //                 Uuid::new_v4(),
                        //                 item.title,
                        //                 parent,
                        //                 profile_id,
                        //             )
                        //             .await?
                        //         }
                        //     };

                        //     model.id
                        // };

                        recurse(db, children, None, profile_id).await?;
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
}
