use colette_core::backup::{BackupRepository, Error};
use colette_netscape::Item;
use colette_opml::Outline;
use deadpool_sqlite::{rusqlite::OptionalExtension, Pool};
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteBackupRepository {
    pool: Pool,
}

impl SqliteBackupRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

struct Parent {
    id: Uuid,
    title: String,
}

#[async_trait::async_trait]
impl BackupRepository for SqliteBackupRepository {
    async fn import_opml(&self, outlines: Vec<Outline>, user_id: Uuid) -> Result<(), Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            let mut stack: Vec<(Option<Parent>, Outline)> = outlines
                .into_iter()
                .map(|outline| (None, outline))
                .collect();

            while let Some((parent, mut outline)) = stack.pop() {
                let title = outline.title.unwrap_or(outline.text);

                if outline.outline.is_some() {
                    let tag_id = {
                        let (mut sql, mut values) =
                            crate::tag::select_by_title(title.clone(), user_id)
                                .build_rusqlite(SqliteQueryBuilder);

                        if let Some(id) = tx
                            .prepare_cached(&sql)?
                            .query_row(&*values.as_params(), |row| row.get::<_, Uuid>("id"))
                            .optional()?
                        {
                            id
                        } else {
                            (sql, values) =
                                crate::tag::insert(Some(Uuid::new_v4()), title.clone(), user_id)
                                    .build_rusqlite(SqliteQueryBuilder);

                            tx.prepare_cached(&sql)?
                                .query_row(&*values.as_params(), |row| row.get::<_, Uuid>("id"))?
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
                            .build_rusqlite(SqliteQueryBuilder);

                        tx.prepare_cached(&sql)?
                            .query_row(&*values.as_params(), |row| row.get::<_, i32>("id"))?
                    };

                    let pf_id = {
                        let (mut sql, mut values) =
                            crate::user_feed::select_by_unique_index(user_id, feed_id)
                                .build_rusqlite(SqliteQueryBuilder);

                        if let Some(id) = tx
                            .prepare_cached(&sql)?
                            .query_row(&*values.as_params(), |row| row.get::<_, Uuid>("id"))
                            .optional()?
                        {
                            id
                        } else {
                            (sql, values) = crate::user_feed::insert(
                                Some(Uuid::new_v4()),
                                None,
                                None,
                                feed_id,
                                user_id,
                            )
                            .build_rusqlite(SqliteQueryBuilder);

                            tx.prepare_cached(&sql)?
                                .query_row(&*values.as_params(), |row| row.get::<_, Uuid>("id"))?
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
                        .build_rusqlite(SqliteQueryBuilder);

                        tx.prepare_cached(&sql)?.execute(&*values.as_params())?;
                    }
                }
            }

            tx.commit()
        })
        .await
        .unwrap()
        .map_err(|e| Error::Unknown(e.into()))
    }

    async fn import_netscape(&self, items: Vec<Item>, user_id: Uuid) -> Result<(), Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

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
                            crate::tag::select_by_title(title.clone(), user_id)
                                .build_rusqlite(SqliteQueryBuilder);

                        if let Some(id) = tx
                            .prepare_cached(&sql)?
                            .query_row(&*values.as_params(), |row| row.get::<_, Uuid>("id"))
                            .optional()?
                        {
                            id
                        } else {
                            (sql, values) =
                                crate::tag::insert(Some(Uuid::new_v4()), title.clone(), user_id)
                                    .build_rusqlite(SqliteQueryBuilder);

                            tx.prepare_cached(&sql)?
                                .query_row(&*values.as_params(), |row| row.get::<_, Uuid>("id"))?
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
                            crate::bookmark::insert(link, item.title, None, None, None)
                                .build_rusqlite(SqliteQueryBuilder);

                        tx.prepare_cached(&sql)?
                            .query_row(&*values.as_params(), |row| row.get::<_, i32>("id"))?
                    };

                    let pb_id = {
                        let (mut sql, mut values) =
                            crate::user_bookmark::select_by_unique_index(user_id, bookmark_id)
                                .build_rusqlite(SqliteQueryBuilder);

                        if let Some(id) = tx
                            .prepare_cached(&sql)?
                            .query_row(&*values.as_params(), |row| row.get::<_, Uuid>("id"))
                            .optional()?
                        {
                            id
                        } else {
                            (sql, values) = crate::user_bookmark::insert(
                                Some(Uuid::new_v4()),
                                bookmark_id,
                                user_id,
                            )
                            .build_rusqlite(SqliteQueryBuilder);

                            tx.prepare_cached(&sql)?
                                .query_row(&*values.as_params(), |row| row.get::<_, Uuid>("id"))?
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
                        .build_rusqlite(SqliteQueryBuilder);

                        tx.prepare_cached(&sql)?.execute(&*values.as_params())?;
                    }
                }
            }

            tx.commit()
        })
        .await
        .unwrap()
        .map_err(|e| Error::Unknown(e.into()))
    }
}
