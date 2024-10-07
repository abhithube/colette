use colette_core::{
    bookmark::{
        BookmarkCreateData, BookmarkFindManyFilters, BookmarkRepository, BookmarkUpdateData,
        Cursor, Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, TagsLinkAction, TagsLinkData, Updatable},
    Bookmark,
};
use sea_orm::{
    prelude::Uuid,
    sqlx::{
        self,
        types::chrono::{DateTime, Utc},
        PgExecutor,
    },
    DatabaseConnection,
};

pub struct BookmarkSqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl BookmarkSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for BookmarkSqlRepository {
    type Params = IdParams;
    type Output = Result<Bookmark, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        find_by_id(self.db.get_postgres_connection_pool(), params).await
    }
}

#[async_trait::async_trait]
impl Creatable for BookmarkSqlRepository {
    type Data = BookmarkCreateData;
    type Output = Result<Bookmark, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut tx = self
            .db
            .get_postgres_connection_pool()
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let bookmark_id = colette_postgres::bookmark::insert(
            &mut *tx,
            data.url,
            data.bookmark.title,
            data.bookmark.thumbnail.map(String::from),
            data.bookmark.published.map(DateTime::<Utc>::from),
            data.bookmark.author,
        )
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

        let pb_id = match colette_postgres::profile_bookmark::select_by_unique_index(
            &mut *tx,
            data.profile_id,
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
                    data.profile_id,
                )
                .await
            }
        }
        .map_err(|e| Error::Unknown(e.into()))?;

        if let Some(tags) = data.tags {
            link_tags(&mut tx, pb_id, tags, data.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        let bookmark = find_by_id(&mut *tx, IdParams::new(pb_id, data.profile_id))
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(bookmark)
    }
}

#[async_trait::async_trait]
impl Updatable for BookmarkSqlRepository {
    type Params = IdParams;
    type Data = BookmarkUpdateData;
    type Output = Result<Bookmark, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut tx = self
            .db
            .get_postgres_connection_pool()
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if let Some(tags) = data.tags {
            link_tags(&mut tx, params.id, tags, params.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        let bookmark = find_by_id(&mut *tx, params).await?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(bookmark)
    }
}

#[async_trait::async_trait]
impl Deletable for BookmarkSqlRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        colette_postgres::profile_bookmark::delete(
            self.db.get_postgres_connection_pool(),
            params.id,
            params.profile_id,
        )
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }
}

#[async_trait::async_trait]
impl BookmarkRepository for BookmarkSqlRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
        filters: Option<BookmarkFindManyFilters>,
    ) -> Result<Vec<Bookmark>, Error> {
        find(
            self.db.get_postgres_connection_pool(),
            None,
            profile_id,
            limit,
            cursor,
            filters,
        )
        .await
    }
}

pub(crate) async fn find(
    executor: impl PgExecutor<'_>,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<BookmarkFindManyFilters>,
) -> Result<Vec<Bookmark>, Error> {
    let mut tags: Option<Vec<String>> = None;

    if let Some(filters) = filters {
        tags = filters.tags;
    }

    colette_postgres::profile_bookmark::find(executor, id, profile_id, tags, cursor, limit)
        .await
        .map_err(|e| Error::Unknown(e.into()))
}

pub async fn find_by_id(
    executor: impl PgExecutor<'_>,
    params: IdParams,
) -> Result<Bookmark, Error> {
    let mut bookmarks = find(
        executor,
        Some(params.id),
        params.profile_id,
        None,
        None,
        None,
    )
    .await?;
    if bookmarks.is_empty() {
        return Err(Error::NotFound(params.id));
    }

    Ok(bookmarks.swap_remove(0))
}

pub(crate) async fn link_tags(
    conn: &mut sqlx::PgConnection,
    profile_bookmark_id: Uuid,
    tags: TagsLinkData,
    profile_id: Uuid,
) -> sqlx::Result<()> {
    if let TagsLinkAction::Remove = tags.action {
        return colette_postgres::profile_bookmark_tag::delete_many_in_titles(
            &mut *conn, &tags.data, profile_id,
        )
        .await;
    }

    if let TagsLinkAction::Set = tags.action {
        colette_postgres::profile_bookmark_tag::delete_many_not_in_titles(
            &mut *conn, &tags.data, profile_id,
        )
        .await?;
    }

    colette_postgres::tag::insert_many(
        &mut *conn,
        tags.data
            .iter()
            .map(|e| colette_postgres::tag::InsertMany {
                id: Uuid::new_v4(),
                title: e.to_owned(),
            })
            .collect(),
        profile_id,
    )
    .await?;

    let tag_ids =
        colette_postgres::tag::select_ids_by_titles(&mut *conn, &tags.data, profile_id).await?;

    let insert_many = tag_ids
        .into_iter()
        .map(|e| colette_postgres::profile_bookmark_tag::InsertMany {
            profile_bookmark_id,
            tag_id: e,
        })
        .collect::<Vec<_>>();

    colette_postgres::profile_bookmark_tag::insert_many(&mut *conn, insert_many, profile_id)
        .await?;

    Ok(())
}
