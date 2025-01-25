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
    async fn import_opml(&self, outlines: Vec<Outline>, user_id: Uuid) -> Result<(), Error> {
        let mut stack: Vec<(Option<Parent>, Outline)> = outlines
            .into_iter()
            .map(|outline| (None, outline))
            .collect();

        while let Some((parent, mut outline)) = stack.pop() {
            let title = outline.title.unwrap_or(outline.text);

            if outline.outline.is_some() {
                let tag_id = {
                    let (mut sql, mut values) = crate::tag::select_by_title(title.clone(), user_id)
                        .build_d1(SqliteQueryBuilder);

                    if let Some(id) = super::first::<Uuid>(&self.db, sql, values, Some("id"))
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    {
                        id
                    } else {
                        let id = Uuid::new_v4();

                        (sql, values) = crate::tag::insert(Some(id), title.clone(), user_id)
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
                    let (sql, values) =
                        crate::feed::insert(Some(Uuid::new_v4()), link, title, outline.xml_url)
                            .build_d1(SqliteQueryBuilder);

                    super::first::<Uuid>(&self.db, sql, values, Some("id"))
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                        .unwrap()
                };

                let pf_id = {
                    let (mut sql, mut values) =
                        crate::user_feed::select_by_unique_index(user_id, feed_id)
                            .build_d1(SqliteQueryBuilder);

                    if let Some(id) = super::first::<Uuid>(&self.db, sql, values, Some("id"))
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    {
                        id
                    } else {
                        let id = Uuid::new_v4();

                        (sql, values) =
                            crate::user_feed::insert(Some(id), None, None, feed_id, user_id)
                                .build_d1(SqliteQueryBuilder);

                        super::run(&self.db, sql, values)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        id
                    }
                };

                if let Some(tag) = parent {
                    let (sql, values) = crate::user_feed_tag::insert_many(
                        &[crate::user_feed_tag::InsertMany {
                            user_feed_id: pf_id,
                            tag_id: tag.id,
                        }],
                        user_id,
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

    async fn import_netscape(&self, items: Vec<Item>, user_id: Uuid) -> Result<(), Error> {
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
                    let (mut sql, mut values) = crate::tag::select_by_title(title.clone(), user_id)
                        .build_d1(SqliteQueryBuilder);

                    if let Some(id) = super::first::<Uuid>(&self.db, sql, values, Some("id"))
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    {
                        id
                    } else {
                        let id = Uuid::new_v4();

                        (sql, values) = crate::tag::insert(Some(id), title.clone(), user_id)
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
                    let (sql, values) = crate::bookmark::insert(
                        Some(Uuid::new_v4()),
                        link,
                        item.title,
                        None,
                        None,
                        None,
                    )
                    .build_d1(SqliteQueryBuilder);

                    super::first::<Uuid>(&self.db, sql, values, Some("id"))
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                        .unwrap()
                };

                let pb_id = {
                    let (mut sql, mut values) =
                        crate::user_bookmark::select_by_unique_index(user_id, bookmark_id)
                            .build_d1(SqliteQueryBuilder);

                    if let Some(id) = super::first::<Uuid>(&self.db, sql, values, Some("id"))
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    {
                        id
                    } else {
                        let id = Uuid::new_v4();

                        (sql, values) = crate::user_bookmark::insert(
                            Some(id),
                            None,
                            None,
                            None,
                            None,
                            None,
                            bookmark_id,
                            user_id,
                        )
                        .build_d1(SqliteQueryBuilder);

                        super::run(&self.db, sql, values)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        id
                    }
                };

                if let Some(tag) = parent {
                    let (sql, values) = crate::user_bookmark_tag::insert_many(
                        &[crate::user_bookmark_tag::InsertMany {
                            user_bookmark_id: pb_id,
                            tag_id: tag.id,
                        }],
                        user_id,
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
