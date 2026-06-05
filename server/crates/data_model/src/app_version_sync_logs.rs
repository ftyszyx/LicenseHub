//! `SeaORM` Entity for application version sync logs.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "app_version_sync_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub app_id: i32,
    pub storage_channel_id: i32,
    pub provider: String,
    pub object_key: String,
    pub public_url: String,
    pub manifest: Json,
    pub status: i16,
    pub error_message: Option<String>,
    pub etag: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub finished_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::apps::Entity",
        from = "Column::AppId",
        to = "super::apps::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Apps,
    #[sea_orm(
        belongs_to = "super::storage_channels::Entity",
        from = "Column::StorageChannelId",
        to = "super::storage_channels::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    StorageChannels,
}

impl Related<super::apps::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Apps.def()
    }
}

impl Related<super::storage_channels::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::StorageChannels.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
