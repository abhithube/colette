use std::sync::Arc;

use colette_core::backup::{BackupRepository, Error};
use colette_netscape::Item;
use colette_opml::Outline;
use sea_query::SqliteQueryBuilder;
use uuid::Uuid;
use worker::D1Database;

use super::D1Binder;

#[derive(Clone)]
pub struct D1BackupRepository {
    db: Arc<D1Database>,
}

impl D1BackupRepository {
    pub fn new(db: Arc<D1Database>) -> Self {
        Self { db }
    }
}

struct Parent {
    id: Uuid,
    title: String,
}

#[async_trait::async_trait]
impl BackupRepository for D1BackupRepository {
    async fn import_opml(&self, outlines: Vec<Outline>, profile_id: Uuid) -> Result<(), Error> {
        let mut stack: Vec<(Option<Parent>, Outline)> = outlines
            .into_iter()
            .map(|outline| (None, outline))
            .collect();

        while let Some((parent, mut outline)) = stack.pop() {
            let title = outline.title.unwrap_or(outline.text);

            if outline.outline.is_some() {
                let tag_id = {
                    let (mut sql, mut values) =
                        colette_sql::tag::select_by_title(title.clone(), profile_id)
                            .build_d1(SqliteQueryBuilder);

                    if let Some(id) = super::first::<Uuid>(&self.db, sql, values, Some("id"))
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    {
                        id
                    } else {
                        let id = Uuid::new_v4();

                        (sql, values) =
                            colette_sql::tag::insert(Some(id), title.clone(), profile_id)
                                .build_d1(SqliteQueryBuilder);

                        super::run(&self.db, sql, values)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        id
                    }
                };

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
            } else if let Some(link) = outline.html_url {
                let feed_id = {
                    let (sql, values) = colette_sql::feed::insert(link, title, outline.xml_url)
                        .build_d1(SqliteQueryBuilder);

                    super::first::<i32>(&self.db, sql, values, Some("id"))
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                        .unwrap()
                };

                let pf_id = {
                    let (mut sql, mut values) =
                        colette_sql::profile_feed::select_by_unique_index(profile_id, feed_id)
                            .build_d1(SqliteQueryBuilder);

                    if let Some(id) = super::first::<Uuid>(&self.db, sql, values, Some("id"))
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    {
                        id
                    } else {
                        let id = Uuid::new_v4();

                        (sql, values) =
                            colette_sql::profile_feed::insert(Some(id), None, feed_id, profile_id)
                                .build_d1(SqliteQueryBuilder);

                        super::run(&self.db, sql, values)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        id
                    }
                };

                if let Some(tag) = parent {
                    let (sql, values) = colette_sql::profile_feed_tag::insert_many(
                        &[colette_sql::profile_feed_tag::InsertMany {
                            profile_feed_id: pf_id,
                            tag_id: tag.id,
                        }],
                        profile_id,
                    )
                    .build_d1(SqliteQueryBuilder);

                    super::run(&self.db, sql, values)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;
                }
            }
        }

        Ok(())
    }

    async fn import_netscape(&self, items: Vec<Item>, profile_id: Uuid) -> Result<(), Error> {
        let mut stack: Vec<(Option<Parent>, Item)> =
            items.into_iter().map(|item| (None, item)).collect();

        while let Some((parent, mut item)) = stack.pop() {
            if item.item.is_some() {
                let title = if let Some(parent) = parent {
                    format!("{}/{}", parent.title, item.title)
                } else {
                    item.title
                };

                let tag_id = {
                    let (mut sql, mut values) =
                        colette_sql::tag::select_by_title(title.clone(), profile_id)
                            .build_d1(SqliteQueryBuilder);

                    if let Some(id) = super::first::<Uuid>(&self.db, sql, values, Some("id"))
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    {
                        id
                    } else {
                        let id = Uuid::new_v4();

                        (sql, values) =
                            colette_sql::tag::insert(Some(id), title.clone(), profile_id)
                                .build_d1(SqliteQueryBuilder);

                        super::run(&self.db, sql, values)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        id
                    }
                };

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
                let bookmark_id = {
                    let (sql, values) =
                        colette_sql::bookmark::insert(link, item.title, None, None, None)
                            .build_d1(SqliteQueryBuilder);

                    super::first::<i32>(&self.db, sql, values, Some("id"))
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                        .unwrap()
                };

                let pb_id = {
                    let (mut sql, mut values) =
                        colette_sql::profile_bookmark::select_by_unique_index(
                            profile_id,
                            bookmark_id,
                        )
                        .build_d1(SqliteQueryBuilder);

                    if let Some(id) = super::first::<Uuid>(&self.db, sql, values, Some("id"))
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    {
                        id
                    } else {
                        let id = Uuid::new_v4();

                        (sql, values) = colette_sql::profile_bookmark::insert(
                            Some(id),
                            bookmark_id,
                            profile_id,
                        )
                        .build_d1(SqliteQueryBuilder);

                        super::run(&self.db, sql, values)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        id
                    }
                };

                if let Some(tag) = parent {
                    let (sql, values) = colette_sql::profile_bookmark_tag::insert_many(
                        &[colette_sql::profile_bookmark_tag::InsertMany {
                            profile_bookmark_id: pb_id,
                            tag_id: tag.id,
                        }],
                        profile_id,
                    )
                    .build_d1(SqliteQueryBuilder);

                    super::run(&self.db, sql, values)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;
                }
            }
        }

        Ok(())
    }
}
