//! `SeaORM` Entity for object storage sync channels.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "storage_channels")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub provider: String,
    pub status: i16,
    pub sort_order: i32,
    pub config: Json,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::app_version_sync_logs::Entity")]
    AppVersionSyncLogs,
}

impl Related<super::app_version_sync_logs::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AppVersionSyncLogs.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
