//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.6

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "feed_entries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    #[sea_orm(column_type = "Text", unique)]
    pub link: String,
    #[sea_orm(column_type = "Text")]
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub published_at: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub author: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub thumbnail_url: Option<String>,
    pub feed_id: i32,
    #[sea_orm(column_type = "Text")]
    pub created_at: String,
    #[sea_orm(column_type = "Text")]
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::feeds::Entity",
        from = "Column::FeedId",
        to = "super::feeds::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Feeds,
    #[sea_orm(has_many = "super::user_feed_entries::Entity")]
    UserFeedEntries,
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

impl ActiveModelBehavior for ActiveModel {}
