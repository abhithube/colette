use colette_core::{
    Tag,
    common::Transaction,
    tag::{Error, TagById, TagCreateData, TagFindParams, TagRepository, TagType, TagUpdateData},
};
use colette_model::{TagWithCounts, bookmark_tags, subscription_tags, tags};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, Iden, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
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
                    subscription_tags::Entity,
                    subscription_tags::Column::SubscriptionId,
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
            .left_join(subscription_tags::Entity)
            .left_join(bookmark_tags::Entity)
            .apply_if(params.ids, |query, ids| {
                query.filter(tags::Column::Id.is_in(ids.into_iter().map(|e| e.to_string())))
            })
            .apply_if(params.user_id, |query, user_id| {
                query.filter(tags::Column::UserId.eq(user_id.to_string()))
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

    async fn find_tag_by_id(&self, tx: &dyn Transaction, id: Uuid) -> Result<TagById, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let Some((id, user_id)) = tags::Entity::find()
            .select_only()
            .columns([tags::Column::Id, tags::Column::UserId])
            .filter(tags::Column::Id.eq(id.to_string()))
            .into_tuple::<(String, String)>()
            .one(tx)
            .await?
        else {
            return Err(Error::NotFound(id));
        };

        Ok(TagById {
            id: id.parse().unwrap(),
            user_id: user_id.parse().unwrap(),
        })
    }

    async fn create_tag(&self, data: TagCreateData) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();
        let model = tags::ActiveModel {
            id: ActiveValue::Set(id.into()),
            title: ActiveValue::Set(data.title.clone()),
            user_id: ActiveValue::Set(data.user_id.into()),
            ..Default::default()
        };
        model.insert(&self.db).await.map_err(|e| match e {
            DbErr::RecordNotInserted => Error::Conflict(data.title),
            _ => Error::Database(e),
        })?;

        Ok(id)
    }

    async fn update_tag(
        &self,
        tx: &dyn Transaction,
        id: Uuid,
        data: TagUpdateData,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let mut model = tags::ActiveModel {
            id: ActiveValue::Unchanged(id.to_string()),
            ..Default::default()
        };

        if let Some(title) = data.title {
            model.title = ActiveValue::Set(title);
        }

        if model.is_changed() {
            model.update(tx).await?;
        }

        Ok(())
    }

    async fn delete_tag(&self, tx: &dyn Transaction, id: Uuid) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        tags::Entity::delete_by_id(id).exec(tx).await?;

        Ok(())
    }
}
