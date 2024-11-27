use colette_core::backup::{BackupRepository, Error};
use colette_netscape::Item;
use colette_opml::Outline;
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteBackupRepository {
    pool: SqlitePool,
}

impl SqliteBackupRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

struct Parent {
    id: Uuid,
    title: String,
}

#[async_trait::async_trait]
impl BackupRepository for SqliteBackupRepository {
    async fn import_opml(&self, outlines: Vec<Outline>, profile_id: Uuid) -> Result<(), Error> {
        let mut tx = self
            .pool
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
                let tag_id = {
                    let (mut sql, mut values) =
                        crate::tag::select_by_title(title.clone(), profile_id)
                            .build_sqlx(SqliteQueryBuilder);

                    if let Some(id) = sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                        .fetch_optional(&mut *tx)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    {
                        id
                    } else {
                        let id = Uuid::new_v4();

                        (sql, values) = crate::tag::insert(Some(id), title.clone(), profile_id)
                            .build_sqlx(SqliteQueryBuilder);

                        sqlx::query_with(&sql, values)
                            .execute(&mut *tx)
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
                    let (sql, values) = crate::feed::insert(link, title, outline.xml_url)
                        .build_sqlx(SqliteQueryBuilder);

                    sqlx::query_scalar_with::<_, i32, _>(&sql, values)
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                };

                let pf_id = {
                    let (mut sql, mut values) =
                        crate::profile_feed::select_by_unique_index(profile_id, feed_id)
                            .build_sqlx(SqliteQueryBuilder);

                    if let Some(id) = sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                        .fetch_optional(&mut *tx)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    {
                        id
                    } else {
                        let id = Uuid::new_v4();

                        (sql, values) =
                            crate::profile_feed::insert(Some(id), None, feed_id, profile_id)
                                .build_sqlx(SqliteQueryBuilder);

                        sqlx::query_with(&sql, values)
                            .execute(&mut *tx)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        id
                    }
                };

                if let Some(tag) = parent {
                    let (sql, values) = crate::profile_feed_tag::insert_many(
                        &[crate::profile_feed_tag::InsertMany {
                            profile_feed_id: pf_id,
                            tag_id: tag.id,
                        }],
                        profile_id,
                    )
                    .build_sqlx(SqliteQueryBuilder);

                    sqlx::query_with(&sql, values)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;
                }
            }
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }

    async fn import_netscape(&self, items: Vec<Item>, profile_id: Uuid) -> Result<(), Error> {
        let mut tx = self
            .pool
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

                let tag_id = {
                    let (mut sql, mut values) =
                        crate::tag::select_by_title(title.clone(), profile_id)
                            .build_sqlx(SqliteQueryBuilder);

                    if let Some(id) = sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                        .fetch_optional(&mut *tx)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    {
                        id
                    } else {
                        let id = Uuid::new_v4();

                        (sql, values) = crate::tag::insert(Some(id), title.clone(), profile_id)
                            .build_sqlx(SqliteQueryBuilder);

                        sqlx::query_with(&sql, values)
                            .execute(&mut *tx)
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
                    let (sql, values) = crate::bookmark::insert(link, item.title, None, None, None)
                        .build_sqlx(SqliteQueryBuilder);

                    sqlx::query_scalar_with::<_, i32, _>(&sql, values)
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                };

                let pb_id = {
                    let (mut sql, mut values) =
                        crate::profile_bookmark::select_by_unique_index(profile_id, bookmark_id)
                            .build_sqlx(SqliteQueryBuilder);

                    if let Some(id) = sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                        .fetch_optional(&mut *tx)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    {
                        id
                    } else {
                        let id = Uuid::new_v4();

                        (sql, values) =
                            crate::profile_bookmark::insert(Some(id), bookmark_id, profile_id)
                                .build_sqlx(SqliteQueryBuilder);

                        sqlx::query_with(&sql, values)
                            .execute(&mut *tx)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        id
                    }
                };

                if let Some(tag) = parent {
                    let (sql, values) = crate::profile_bookmark_tag::insert_many(
                        &[crate::profile_bookmark_tag::InsertMany {
                            profile_bookmark_id: pb_id,
                            tag_id: tag.id,
                        }],
                        profile_id,
                    )
                    .build_sqlx(SqliteQueryBuilder);

                    sqlx::query_with(&sql, values)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;
                }
            }
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }
}
