//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.6

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user_feeds")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    #[sea_orm(column_type = "Text")]
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub user_id: String,
    pub feed_id: i32,
    pub created_at: i32,
    pub updated_at: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::feeds::Entity",
        from = "Column::FeedId",
        to = "super::feeds::Column::Id",
        on_update = "NoAction",
        on_delete = "Restrict"
    )]
    Feeds,
    #[sea_orm(has_many = "super::user_feed_entries::Entity")]
    UserFeedEntries,
    #[sea_orm(has_many = "super::user_feed_tags::Entity")]
    UserFeedTags,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Users,
}

impl Related<super::feeds::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Feeds.def()
    }
}

impl Related<super::user_feed_entries::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserFeedEntries.def()
    }
}

impl Related<super::user_feed_tags::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserFeedTags.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
