use colette_core::tags::{
    Error, TagType, TagsCreateData, TagsFindManyParams, TagsFindOneParams, TagsRepository,
    TagsUpdateData,
};
use colette_entities::tag;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set, SqlErr,
    TransactionError, TransactionTrait,
};
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

    async fn find_one_tag(&self, params: TagsFindOneParams) -> Result<colette_core::Tag, Error> {
        sqlx::query_file_as!(
            Tag,
            "queries/tags/find_one.sql",
            params.slug.clone(),
            params.profile_id
        )
        .fetch_one(self.db.get_postgres_connection_pool())
        .await
        .map(colette_core::Tag::from)
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.slug),
            _ => Error::Unknown(e.into()),
        })
    }

    async fn create_tag(&self, data: TagsCreateData) -> Result<colette_core::Tag, Error> {
        let model = tag::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(data.title.clone()),
            slug: Set(slug::slugify(data.title.clone())),
            profile_id: Set(data.profile_id),
            ..Default::default()
        };

        let tag = tag::Entity::insert(model)
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| match e.sql_err() {
                Some(SqlErr::UniqueConstraintViolation(_)) => Error::Conflict(data.title),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(Tag {
            id: tag.id,
            title: tag.title,
            slug: tag.slug,
            bookmark_count: Some(0),
            feed_count: Some(0),
        }
        .into())
    }

    async fn update_tag(
        &self,
        params: TagsFindOneParams,
        data: TagsUpdateData,
    ) -> Result<colette_core::Tag, Error> {
        self.db
            .transaction::<_, (), Error>(|txn| {
                let params = params.clone();
                Box::pin(async move {
                    let Some(model) = tag::Entity::find()
                        .filter(tag::Column::Slug.eq(params.slug.clone()))
                        .filter(tag::Column::ProfileId.eq(params.profile_id))
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.slug));
                    };
                    let mut active_model = model.into_active_model();

                    if let Some(title) = data.title {
                        active_model.title.set_if_not_equals(title);
                    }

                    if active_model.is_changed() {
                        active_model
                            .update(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })?;

        self.find_one_tag(params).await
    }

    async fn delete_tag(&self, params: TagsFindOneParams) -> Result<(), Error> {
        let result = tag::Entity::delete_many()
            .filter(tag::Column::Slug.eq(params.slug.clone()))
            .filter(tag::Column::ProfileId.eq(params.profile_id))
            .exec(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(params.slug));
        }

        Ok(())
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub(crate) struct Tag {
    id: Uuid,
    title: String,
    slug: String,
    bookmark_count: Option<i64>,
    feed_count: Option<i64>,
}

impl From<Tag> for colette_core::Tag {
    fn from(value: Tag) -> Self {
        Self {
            id: value.id,
            title: value.title,
            slug: value.slug,
            bookmark_count: value.bookmark_count,
            feed_count: value.feed_count,
        }
    }
}
