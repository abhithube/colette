use chrono::{DateTime, Utc};
use colette_core::backup::{BackupRepository, Error};
use colette_netscape::Item;
use colette_opml::Outline;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresBackupRepository {
    pool: Pool<Postgres>,
}

impl PostgresBackupRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

struct Parent {
    id: Uuid,
    title: String,
}

#[async_trait::async_trait]
impl BackupRepository for PostgresBackupRepository {
    async fn import_opml(&self, outlines: Vec<Outline>, user_id: Uuid) -> Result<(), Error> {
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
                    if let Some(id) =
                        sqlx::query_file_scalar!("queries/tags/select_by_title.sql", title, user_id)
                            .fetch_optional(&mut *tx)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                    {
                        id
                    } else {
                        sqlx::query_file_scalar!("queries/tags/insert.sql", title, user_id)
                            .fetch_one(&mut *tx)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
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
                let feed_id = sqlx::query_file_scalar!(
                    "queries/feeds/insert.sql",
                    link,
                    title,
                    outline.xml_url
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

                let pf_id = {
                    if let Some(id) = sqlx::query_file_scalar!(
                        "queries/user_feeds/select_by_index.sql",
                        user_id,
                        feed_id
                    )
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
                    {
                        id
                    } else {
                        sqlx::query_file_scalar!(
                            "queries/user_feeds/insert.sql",
                            Option::<&str>::None,
                            Option::<Uuid>::None,
                            feed_id,
                            user_id
                        )
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    }
                };

                // if let Some(tag) = parent {
                //     let (sql, values) = crate::query::user_feed_tag::insert_many(
                //         &[crate::query::user_feed_tag::InsertMany {
                //             user_feed_id: pf_id,
                //             tag_id: tag.id,
                //         }],
                //         user_id,
                //     )
                //     .build_sqlx(PostgresQueryBuilder);

                //     tx.execute(&stmt, &values.as_params())
                //         .await
                //         .map_err(|e| Error::Unknown(e.into()))?;
                // }
            }
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }

    async fn import_netscape(&self, items: Vec<Item>, user_id: Uuid) -> Result<(), Error> {
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
                    if let Some(id) =
                        sqlx::query_file_scalar!("queries/tags/select_by_title.sql", title, user_id)
                            .fetch_optional(&mut *tx)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                    {
                        id
                    } else {
                        sqlx::query_file_scalar!("queries/tags/insert.sql", title, user_id)
                            .fetch_one(&mut *tx)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
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
                let bookmark_id = sqlx::query_file_scalar!(
                    "queries/bookmarks/insert.sql",
                    link,
                    item.title,
                    Option::<&str>::None,
                    Option::<DateTime<Utc>>::None,
                    Option::<&str>::None
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

                let pb_id = {
                    if let Some(id) = sqlx::query_file_scalar!(
                        "queries/user_bookmarks/select_by_index.sql",
                        user_id,
                        bookmark_id
                    )
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
                    {
                        id
                    } else {
                        sqlx::query_file_scalar!(
                            "queries/user_bookmarks/insert.sql",
                            Option::<&str>::None,
                            Option::<&str>::None,
                            Option::<DateTime<Utc>>::None,
                            Option::<&str>::None,
                            Option::<Uuid>::None,
                            bookmark_id,
                            user_id
                        )
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    }
                };

                // if let Some(tag) = parent {
                //     let (sql, values) = crate::query::user_bookmark_tag::insert_many(
                //         &[crate::query::user_bookmark_tag::InsertMany {
                //             user_bookmark_id: pb_id,
                //             tag_id: tag.id,
                //         }],
                //         user_id,
                //     )
                //     .build_sqlx(PostgresQueryBuilder);

                //     tx.execute(&stmt, &values.as_params())
                //         .await
                //         .map_err(|e| Error::Unknown(e.into()))?;
                // }
            }
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }
}
