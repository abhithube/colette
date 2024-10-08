use colette_core::backup::{BackupRepository, Error};
use colette_netscape::Item;
use colette_opml::Outline;
use sea_orm::{prelude::Uuid, DatabaseConnection};

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
        let mut tx = self
            .db
            .get_postgres_connection_pool()
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let mut stack: Vec<(Option<Parent>, Outline)> = outlines
            .into_iter()
            .map(|outline| (None, outline))
            .collect();

        while let Some((parent, mut outline)) = stack.pop() {
            let title = outline.title.unwrap_or(outline.text);

            if outline.outline.is_some() {
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

                if let Some(children) = outline.outline.take() {
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
            } else if let Some(link) = outline.xml_url {
                let feed_id =
                    colette_postgres::feed::insert(&mut *tx, link, title, outline.html_url)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                let profile_feed_id = match colette_postgres::profile_feed::select_by_unique_index(
                    &mut *tx, profile_id, feed_id,
                )
                .await
                {
                    Ok(id) => Ok(id),
                    _ => {
                        colette_postgres::profile_feed::insert(
                            &mut *tx,
                            Uuid::new_v4(),
                            None,
                            feed_id,
                            profile_id,
                        )
                        .await
                    }
                }
                .map_err(|e| Error::Unknown(e.into()))?;

                if let Some(tag) = parent {
                    colette_postgres::profile_feed_tag::insert_many(
                        &mut *tx,
                        vec![colette_postgres::profile_feed_tag::InsertMany {
                            profile_feed_id,
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
