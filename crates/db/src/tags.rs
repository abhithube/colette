use colette_core::{
    common::FindOneParams,
    tags::{Error, TagType, TagsCreateData, TagsFindManyParams, TagsRepository, TagsUpdateData},
};
use colette_entities::tag;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::PostgresRepository;

#[async_trait::async_trait]
impl TagsRepository for PostgresRepository {
    async fn find_many_tags(
        &self,
        params: TagsFindManyParams,
    ) -> Result<Vec<colette_core::Tag>, Error> {
        match params.tag_type {
            TagType::All => {
                sqlx::query_file_as!(Tag, "queries/tags/find_many.sql", params.profile_id)
                    .fetch_all(self.db.get_postgres_connection_pool())
                    .await
            }
            TagType::Bookmarks => {
                sqlx::query_file_as!(
                    Tag,
                    "queries/tags/find_many_profile_bookmark_tags.sql",
                    params.profile_id
                )
                .fetch_all(self.db.get_postgres_connection_pool())
                .await
            }
            TagType::Feeds => {
                sqlx::query_file_as!(
                    Tag,
                    "queries/tags/find_many_profile_feed_tags.sql",
                    params.profile_id
                )
                .fetch_all(self.db.get_postgres_connection_pool())
                .await
            }
        }
        .map(|e| e.into_iter().map(colette_core::Tag::from).collect())
        .map_err(|e| Error::Unknown(e.into()))
    }

    async fn find_one_tag(&self, params: FindOneParams) -> Result<colette_core::Tag, Error> {
        sqlx::query_file_as!(
            Tag,
            "queries/tags/find_one.sql",
            params.id,
            params.profile_id
        )
        .fetch_one(self.db.get_postgres_connection_pool())
        .await
        .map(colette_core::Tag::from)
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }

    async fn create_tag(&self, data: TagsCreateData) -> Result<colette_core::Tag, Error> {
        sqlx::query_file_as!(
            Tag,
            "queries/tags/insert.sql",
            Uuid::new_v4(),
            data.title,
            data.profile_id
        )
        .fetch_one(self.db.get_postgres_connection_pool())
        .await
        .map(colette_core::Tag::from)
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(data.title),
            _ => Error::Unknown(e.into()),
        })
    }

    async fn update_tag(
        &self,
        params: FindOneParams,
        data: TagsUpdateData,
    ) -> Result<colette_core::Tag, Error> {
        sqlx::query_file_as!(
            Tag,
            "queries/tags/update.sql",
            params.id,
            params.profile_id,
            data.title
        )
        .fetch_one(self.db.get_postgres_connection_pool())
        .await
        .map(colette_core::Tag::from)
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }

    async fn delete_tag(&self, params: FindOneParams) -> Result<(), Error> {
        let result = tag::Entity::delete_by_id(params.id)
            .filter(tag::Column::ProfileId.eq(params.profile_id))
            .exec(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub(crate) struct Tag {
    id: Uuid,
    title: String,
    bookmark_count: Option<i64>,
    feed_count: Option<i64>,
}

impl From<Tag> for colette_core::Tag {
    fn from(value: Tag) -> Self {
        Self {
            id: value.id,
            title: value.title,
            bookmark_count: value.bookmark_count,
            feed_count: value.feed_count,
        }
    }
}
