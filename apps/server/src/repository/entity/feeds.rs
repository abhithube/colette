//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.6

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "feeds")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    #[sea_orm(column_type = "Text", unique)]
    pub link: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub xml_url: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub created_at: String,
    #[sea_orm(column_type = "Text")]
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::feed_entries::Entity")]
    FeedEntries,
    #[sea_orm(has_many = "super::user_feeds::Entity")]
    UserFeeds,
}

impl Related<super::feed_entries::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FeedEntries.def()
    }
}

impl Related<super::user_feeds::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserFeeds.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
