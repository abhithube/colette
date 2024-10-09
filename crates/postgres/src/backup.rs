use colette_core::backup::{BackupRepository, Error};
use colette_netscape::Item;
use colette_opml::Outline;
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

pub struct PostgresBackupRepository {
    pub(crate) pool: Pool,
}

impl PostgresBackupRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

struct Parent {
    id: Uuid,
    title: String,
}

#[async_trait::async_trait]
impl BackupRepository for PostgresBackupRepository {
    async fn import_opml(&self, outlines: Vec<Outline>, profile_id: Uuid) -> Result<(), Error> {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
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
                        colette_sql::tag::select_by_title(title.clone(), profile_id)
                            .build_postgres(PostgresQueryBuilder);

                    if let Some(row) = tx
                        .query_opt(&sql, &values.as_params())
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    {
                        row.get("id")
                    } else {
                        (sql, values) =
                            colette_sql::tag::insert(Uuid::new_v4(), title.clone(), profile_id)
                                .build_postgres(PostgresQueryBuilder);

                        let row = tx
                            .query_one(&sql, &values.as_params())
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        row.get("id")
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
                        .build_postgres(PostgresQueryBuilder);

                    let row = tx
                        .query_one(&sql, &values.as_params())
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    row.get::<_, i32>("id")
                };

                let pf_id = {
                    let (mut sql, mut values) =
                        colette_sql::profile_feed::select_by_unique_index(profile_id, feed_id)
                            .build_postgres(PostgresQueryBuilder);

                    if let Some(row) = tx
                        .query_opt(&sql, &values.as_params())
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    {
                        row.get("id")
                    } else {
                        (sql, values) = colette_sql::profile_feed::insert(
                            Uuid::new_v4(),
                            None,
                            feed_id,
                            profile_id,
                        )
                        .build_postgres(PostgresQueryBuilder);

                        let row = tx
                            .query_one(&sql, &values.as_params())
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        row.get("id")
                    }
                };

                if let Some(tag) = parent {
                    let (sql, values) = colette_sql::profile_feed_tag::insert_many(
                        vec![colette_sql::profile_feed_tag::InsertMany {
                            profile_feed_id: pf_id,
                            tag_id: tag.id,
                        }],
                        profile_id,
                    )
                    .build_postgres(PostgresQueryBuilder);

                    tx.execute(&sql, &values.as_params())
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;
                }
            }
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }

    async fn import_netscape(&self, items: Vec<Item>, profile_id: Uuid) -> Result<(), Error> {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
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
                        colette_sql::tag::select_by_title(title.clone(), profile_id)
                            .build_postgres(PostgresQueryBuilder);

                    if let Some(row) = tx
                        .query_opt(&sql, &values.as_params())
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    {
                        row.get("id")
                    } else {
                        (sql, values) =
                            colette_sql::tag::insert(Uuid::new_v4(), title.clone(), profile_id)
                                .build_postgres(PostgresQueryBuilder);

                        let row = tx
                            .query_one(&sql, &values.as_params())
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        row.get("id")
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
                            .build_postgres(PostgresQueryBuilder);

                    let row = tx
                        .query_one(&sql, &values.as_params())
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    row.get::<_, i32>("id")
                };

                let pb_id = {
                    let (mut sql, mut values) =
                        colette_sql::profile_bookmark::select_by_unique_index(
                            profile_id,
                            bookmark_id,
                        )
                        .build_postgres(PostgresQueryBuilder);

                    if let Some(row) = tx
                        .query_opt(&sql, &values.as_params())
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    {
                        row.get("id")
                    } else {
                        (sql, values) = colette_sql::profile_bookmark::insert(
                            Uuid::new_v4(),
                            bookmark_id,
                            profile_id,
                        )
                        .build_postgres(PostgresQueryBuilder);

                        let row = tx
                            .query_one(&sql, &values.as_params())
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        row.get("id")
                    }
                };

                if let Some(tag) = parent {
                    let (sql, values) = colette_sql::profile_bookmark_tag::insert_many(
                        vec![colette_sql::profile_bookmark_tag::InsertMany {
                            profile_bookmark_id: pb_id,
                            tag_id: tag.id,
                        }],
                        profile_id,
                    )
                    .build_postgres(PostgresQueryBuilder);

                    tx.execute(&sql, &values.as_params())
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;
                }
            }
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }
}
