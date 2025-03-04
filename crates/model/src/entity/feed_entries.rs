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
    pub published_at: i32,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub author: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub thumbnail_url: Option<String>,
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
        on_delete = "Cascade"
    )]
    Feeds,
    #[sea_orm(has_many = "super::subscription_entries::Entity")]
    SubscriptionEntries,
}

impl Related<super::feeds::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Feeds.def()
    }
}

impl Related<super::subscription_entries::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SubscriptionEntries.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
