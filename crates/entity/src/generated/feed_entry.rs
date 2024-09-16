//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "feed_entry")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text", unique)]
    pub link: String,
    #[sea_orm(column_type = "Text")]
    pub title: String,
    pub published_at: DateTimeWithTimeZone,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub author: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub thumbnail_url: Option<String>,
    pub feed_id: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::feed::Entity",
        from = "Column::FeedId",
        to = "super::feed::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Feed,
    #[sea_orm(has_many = "super::profile_feed_entry::Entity")]
    ProfileFeedEntry,
}

impl Related<super::feed::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Feed.def()
    }
}

impl Related<super::profile_feed_entry::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProfileFeedEntry.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
