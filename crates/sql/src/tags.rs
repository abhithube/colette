use colette_core::{
    common::{Paginated, PAGINATION_LIMIT},
    tags::{Error, TagType, TagsCreateData, TagsFindManyFilters, TagsRepository, TagsUpdateData},
    Tag,
};
use colette_entities::{profile_bookmark_tag, profile_feed_tag, tag, PartialTag};
use sea_orm::{
    sea_query::{Alias, Expr},
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, JoinType,
    QueryFilter, QuerySelect, RelationTrait, Set, SqlErr, TransactionError, TransactionTrait,
};
use uuid::Uuid;

use crate::{utils, SqlRepository};

#[async_trait::async_trait]
impl TagsRepository for SqlRepository {
    async fn find_many_tags(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor_raw: Option<String>,
        filters: TagsFindManyFilters,
    ) -> Result<Paginated<Tag>, Error> {
        let mut cursor = Cursor::default();
        if let Some(raw) = cursor_raw.as_deref() {
            cursor = utils::decode_cursor::<Cursor>(raw).map_err(|e| Error::Unknown(e.into()))?;
        }

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
            .filter(tag::Column::ProfileId.eq(profile_id))
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

        query = match filters.tag_type {
            TagType::Bookmarks => {
                query.join(JoinType::InnerJoin, tag::Relation::ProfileBookmarkTag.def())
            }
            TagType::Feeds => query.join(JoinType::InnerJoin, tag::Relation::ProfileFeedTag.def()),
            _ => query,
        };

        let mut query = query.cursor_by(tag::Column::Title);

        query.after(cursor.title);
        if let Some(limit) = limit {
            query.first(limit);
        }

        let mut tags = query
            .into_model::<PartialTag>()
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Tag::from).collect::<Vec<_>>())
            .map_err(|e| Error::Unknown(e.into()))?;
        let mut cursor: Option<String> = None;

        if tags.len() > PAGINATION_LIMIT {
            tags = tags.into_iter().take(PAGINATION_LIMIT).collect();

            if let Some(last) = tags.last() {
                let c = Cursor {
                    title: last.title.to_owned(),
                };
                let encoded = utils::encode_cursor(&c).map_err(|e| Error::Unknown(e.into()))?;

                cursor = Some(encoded);
            }
        }

        Ok(Paginated::<Tag> { cursor, data: tags })
    }

    async fn find_one_tag(&self, id: Uuid, profile_id: Uuid) -> Result<Tag, Error> {
        find_by_id(&self.db, id, profile_id).await
    }

    async fn create_tag(&self, data: TagsCreateData) -> Result<Tag, Error> {
        let model = tag::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(data.title.clone()),
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
            bookmark_count: Some(0),
            feed_count: Some(0),
        })
    }

    async fn update_tag(
        &self,
        id: Uuid,
        profile_id: Uuid,
        data: TagsUpdateData,
    ) -> Result<Tag, Error> {
        self.db
            .transaction::<_, Tag, Error>(|txn| {
                Box::pin(async move {
                    let Some(model) = tag::Entity::find_by_id(id)
                        .filter(tag::Column::ProfileId.eq(profile_id))
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(id));
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

                    find_by_id(txn, id, profile_id).await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn delete_tag(&self, id: Uuid, profile_id: Uuid) -> Result<(), Error> {
        let result = tag::Entity::delete_by_id(id)
            .filter(tag::Column::ProfileId.eq(profile_id))
            .exec(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(id));
        }

        Ok(())
    }
}

async fn find_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
) -> Result<Tag, Error> {
    let Some(tag) = tag::Entity::find_by_id(id)
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
        .filter(tag::Column::ProfileId.eq(profile_id))
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
        return Err(Error::NotFound(id));
    };

    Ok(tag.into())
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
struct Cursor {
    pub title: String,
}
