use colette_core::{
    Tag,
    common::IdParams,
    tag::{Error, TagCreateData, TagFindParams, TagRepository, TagType, TagUpdateData},
};
use colette_model::{TagWithCounts, bookmark_tags, tags, user_feed_tags};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, Iden,
    IntoActiveModel, ModelTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
    TransactionTrait,
    prelude::Expr,
    sea_query::{Alias, Func},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteTagRepository {
    db: DatabaseConnection,
}

impl SqliteTagRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl TagRepository for SqliteTagRepository {
    async fn find_tags(&self, params: TagFindParams) -> Result<Vec<Tag>, Error> {
        let feed_count = Alias::new("feed_count");
        let bookmark_count = Alias::new("bookmark_count");

        let tags = tags::Entity::find()
            .expr_as(
                Func::count(Expr::col((
                    user_feed_tags::Entity,
                    user_feed_tags::Column::UserFeedId,
                ))),
                feed_count.to_string(),
            )
            .expr_as(
                Func::count(Expr::col((
                    bookmark_tags::Entity,
                    bookmark_tags::Column::BookmarkId,
                ))),
                bookmark_count.to_string(),
            )
            .filter(tags::Column::UserId.eq(params.user_id.to_string()))
            .left_join(user_feed_tags::Entity)
            .left_join(bookmark_tags::Entity)
            .apply_if(params.id, |query, id| {
                query.filter(tags::Column::Id.eq(id.to_string()))
            })
            .apply_if(params.cursor, |query, cursor| {
                query.filter(tags::Column::Title.gt(cursor.title))
            })
            .group_by(tags::Column::Id)
            .apply_if(
                if params.tag_type == TagType::Feeds {
                    Some(true)
                } else {
                    None
                },
                |query, _| query.having(Expr::col(feed_count).gt(Expr::val(0))),
            )
            .apply_if(
                if params.tag_type == TagType::Bookmarks {
                    Some(true)
                } else {
                    None
                },
                |query, _| query.having(Expr::col(bookmark_count).gt(Expr::val(0))),
            )
            .order_by_asc(tags::Column::Title)
            .limit(params.limit.map(|e| e as u64))
            .into_model::<TagWithCounts>()
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(tags)
    }

    async fn create_tag(&self, data: TagCreateData) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();
        let tag = tags::ActiveModel {
            id: ActiveValue::Set(id.into()),
            title: ActiveValue::Set(data.title.clone()),
            user_id: ActiveValue::Set(data.user_id.into()),
            ..Default::default()
        };
        tag.insert(&self.db).await.map_err(|e| match e {
            DbErr::RecordNotInserted => Error::Conflict(data.title),
            _ => Error::Database(e),
        })?;

        Ok(id)
    }

    async fn update_tag(&self, params: IdParams, data: TagUpdateData) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let Some(tag) = tags::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if tag.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        let mut tag = tag.into_active_model();

        if let Some(title) = data.title {
            tag.title = ActiveValue::Set(title);
        }

        if tag.is_changed() {
            tag.update(&tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn delete_tag(&self, params: IdParams) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let Some(tag) = tags::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if tag.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        tag.delete(&tx).await?;

        tx.commit().await?;

        Ok(())
    }
}
