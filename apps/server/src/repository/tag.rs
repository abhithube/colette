use colette_core::{
    Tag,
    common::Transaction,
    tag::{
        Error, TagById, TagCreateParams, TagDeleteParams, TagFindByIdsParams, TagFindParams,
        TagRepository, TagType, TagUpdateParams,
    },
};
use colette_model::{TagWithCounts, bookmark_tags, subscription_tags, tags};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr, FromQueryResult,
    sea_query::{Alias, Expr, Func, Order, Query},
};

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

        let mut query = Query::select()
            .columns([
                (tags::Entity, tags::Column::Id),
                (tags::Entity, tags::Column::Title),
                (tags::Entity, tags::Column::UserId),
                (tags::Entity, tags::Column::CreatedAt),
                (tags::Entity, tags::Column::UpdatedAt),
            ])
            .expr_as(
                Func::count(Expr::col((
                    subscription_tags::Entity,
                    subscription_tags::Column::SubscriptionId,
                ))),
                feed_count.clone(),
            )
            .expr_as(
                Func::count(Expr::col((
                    bookmark_tags::Entity,
                    bookmark_tags::Column::BookmarkId,
                ))),
                bookmark_count.clone(),
            )
            .from(tags::Entity)
            .left_join(
                subscription_tags::Entity,
                Expr::col((subscription_tags::Entity, subscription_tags::Column::TagId))
                    .eq(Expr::col((tags::Entity, tags::Column::Id))),
            )
            .left_join(
                bookmark_tags::Entity,
                Expr::col((bookmark_tags::Entity, bookmark_tags::Column::TagId))
                    .eq(Expr::col((tags::Entity, tags::Column::Id))),
            )
            .apply_if(params.ids, |query, ids| {
                query.and_where(
                    Expr::col((tags::Entity, tags::Column::Id))
                        .is_in(ids.into_iter().map(String::from)),
                );
            })
            .apply_if(params.user_id, |query, user_id| {
                query.and_where(
                    Expr::col((tags::Entity, tags::Column::UserId)).eq(user_id.to_string()),
                );
            })
            .apply_if(params.cursor, |query, cursor| {
                query.and_where(
                    Expr::col((tags::Entity, tags::Column::Title)).gt(Expr::val(cursor.title)),
                );
            })
            .group_by_col((tags::Entity, tags::Column::Id))
            .order_by((tags::Entity, tags::Column::CreatedAt), Order::Asc)
            .to_owned();

        match params.tag_type {
            TagType::Feeds => {
                query.and_having(Expr::col(feed_count).gt(Expr::val(0)));
            }
            TagType::Bookmarks => {
                query.and_having(Expr::col(bookmark_count).gt(Expr::val(0)));
            }
            _ => {}
        }

        if let Some(limit) = params.limit {
            query.limit(limit as u64);
        }

        let tags = TagWithCounts::find_by_statement(self.db.get_database_backend().build(&query))
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(tags)
    }

    async fn find_tags_by_ids(
        &self,
        tx: &dyn Transaction,
        params: TagFindByIdsParams,
    ) -> Result<Vec<TagById>, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let query = Query::select()
            .column((tags::Entity, tags::Column::Id))
            .column((tags::Entity, tags::Column::UserId))
            .from(tags::Entity)
            .and_where(
                Expr::col((tags::Entity, tags::Column::Id))
                    .is_in(params.ids.into_iter().map(String::from)),
            )
            .to_owned();

        let tags = tx
            .query_all(self.db.get_database_backend().build(&query))
            .await
            .map(|e| {
                e.into_iter()
                    .map(|e| TagById {
                        id: e.try_get_by_index::<String>(0).unwrap().parse().unwrap(),
                        user_id: e.try_get_by_index::<String>(1).unwrap().parse().unwrap(),
                    })
                    .collect()
            })?;

        Ok(tags)
    }

    async fn create_tag(&self, params: TagCreateParams) -> Result<(), Error> {
        let query = Query::insert()
            .columns([tags::Column::Id, tags::Column::Title, tags::Column::UserId])
            .values_panic([
                params.id.to_string().into(),
                params.title.clone().into(),
                params.user_id.to_string().into(),
            ])
            .to_owned();

        self.db
            .execute(self.db.get_database_backend().build(&query))
            .await
            .map_err(|e| match e {
                DbErr::RecordNotInserted => Error::Conflict(params.title),
                _ => Error::Database(e),
            })?;

        Ok(())
    }

    async fn update_tag(&self, tx: &dyn Transaction, params: TagUpdateParams) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        if params.title.is_none() {
            return Ok(());
        }

        let mut query = Query::update()
            .table(tags::Entity)
            .and_where(Expr::col(tags::Column::Id).eq(params.id.to_string()))
            .to_owned();

        if let Some(title) = params.title {
            query.value(tags::Column::Title, title);
        }

        tx.execute(self.db.get_database_backend().build(&query))
            .await?;

        Ok(())
    }

    async fn delete_tag(&self, tx: &dyn Transaction, params: TagDeleteParams) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let query = Query::delete()
            .from_table(tags::Entity)
            .and_where(Expr::col(tags::Column::Id).eq(params.id.to_string()))
            .to_owned();

        tx.execute(self.db.get_database_backend().build(&query))
            .await?;

        Ok(())
    }
}
