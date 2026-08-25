//! `SeaORM` Entity for managed storage resources.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "resources")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub storage_channel_id: i32,
    #[sea_orm(unique)]
    pub object_key: String,
    pub resource_type: String,
    pub original_name: String,
    pub content_type: String,
    pub size: i64,
    pub uploaded_by: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::storage_channels::Entity",
        from = "Column::StorageChannelId",
        to = "super::storage_channels::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    StorageChannels,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UploadedBy",
        to = "super::users::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    Users,
}

impl Related<super::storage_channels::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::StorageChannels.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
