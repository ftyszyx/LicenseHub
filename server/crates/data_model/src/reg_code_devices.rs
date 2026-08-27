use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "reg_code_devices")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub reg_code_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub device_id: i32,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::app_devices::Entity",
        from = "Column::DeviceId",
        to = "super::app_devices::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    AppDevices,
    #[sea_orm(
        belongs_to = "super::reg_codes::Entity",
        from = "Column::RegCodeId",
        to = "super::reg_codes::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    RegCodes,
}

impl Related<super::app_devices::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AppDevices.def()
    }
}

impl Related<super::reg_codes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RegCodes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
