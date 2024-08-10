use colette_core::{
    tags::{
        Error, TagType, TagsCreateData, TagsFindManyParams, TagsFindOneParams, TagsRepository,
        TagsUpdateData,
    },
    Tag,
};
use colette_entities::{profile_bookmark_tag, profile_feed_tag, tag, PartialTag};
use sea_orm::{
    sea_query::{Alias, Expr},
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, JoinType,
    QueryFilter, QuerySelect, RelationTrait, Set, SqlErr, TransactionError, TransactionTrait,
};
use uuid::Uuid;

use crate::PostgresRepository;

#[async_trait::async_trait]
impl TagsRepository for PostgresRepository {
    async fn find_many_tags(&self, params: TagsFindManyParams) -> Result<Vec<Tag>, Error> {
        let mut query = tag::Entity::find()
            .expr_as(
                Expr::col((
                    Alias::new("pbt"),
                    profile_bookmark_tag::Column::ProfileBookmarkId,
                ))
                .count(),
                "bookmark_count",
            )
            .expr_as(
                Expr::col((Alias::new("pft"), profile_feed_tag::Column::ProfileFeedId)).count(),
                "feed_count",
            )
            .filter(tag::Column::ProfileId.eq(params.profile_id))
            .join_as(
                JoinType::LeftJoin,
                tag::Relation::ProfileBookmarkTag.def(),
                Alias::new("pbt"),
            )
            .join_as(
                JoinType::LeftJoin,
                tag::Relation::ProfileFeedTag.def(),
                Alias::new("pft"),
            )
            .group_by(tag::Column::Id);

        query = match params.tag_type {
            TagType::Bookmarks => {
                query.join(JoinType::InnerJoin, tag::Relation::ProfileBookmarkTag.def())
            }
            TagType::Feeds => query.join(JoinType::InnerJoin, tag::Relation::ProfileFeedTag.def()),
            _ => query,
        };

        let tags = query
            .into_model::<PartialTag>()
            .all(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(tags.into_iter().map(Tag::from).collect::<Vec<_>>())
    }

    async fn find_one_tag(&self, params: TagsFindOneParams) -> Result<Tag, Error> {
        find_by_slug(&self.db, params).await
    }

    async fn create_tag(&self, data: TagsCreateData) -> Result<Tag, Error> {
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
        })
    }

    async fn update_tag(
        &self,
        params: TagsFindOneParams,
        data: TagsUpdateData,
    ) -> Result<Tag, Error> {
        self.db
            .transaction::<_, Tag, Error>(|txn| {
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

                    find_by_slug(txn, params).await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
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

async fn find_by_slug<Db: ConnectionTrait>(
    db: &Db,
    params: TagsFindOneParams,
) -> Result<Tag, Error> {
    let Some(tag) = tag::Entity::find()
        .expr_as(
            Expr::col((
                Alias::new("pbt"),
                profile_bookmark_tag::Column::ProfileBookmarkId,
            ))
            .count(),
            "bookmark_count",
        )
        .expr_as(
            Expr::col((Alias::new("pft"), profile_feed_tag::Column::ProfileFeedId)).count(),
            "feed_count",
        )
        .filter(tag::Column::Slug.eq(params.slug.clone()))
        .filter(tag::Column::ProfileId.eq(params.profile_id))
        .join_as(
            JoinType::LeftJoin,
            tag::Relation::ProfileBookmarkTag.def(),
            Alias::new("pbt"),
        )
        .join_as(
            JoinType::LeftJoin,
            tag::Relation::ProfileFeedTag.def(),
            Alias::new("pft"),
        )
        .group_by(tag::Column::Id)
        .into_model::<PartialTag>()
        .one(db)
        .await
        .map_err(|e| Error::Unknown(e.into()))?
    else {
        return Err(Error::NotFound(params.slug));
    };

    Ok(tag.into())
}
