//! `SeaORM` Entity for payment orders.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "orders")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub order_no: String,
    pub plan_id: i32,
    pub app_id: i32,
    pub amount_cents: i32,
    pub pay_type: String,
    pub status: i16,
    pub provider: String,
    pub provider_trade_no: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub pay_url: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub qr_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub url_scheme: Option<String>,
    pub reg_code_id: Option<i32>,
    pub client_ip: Option<String>,
    pub provider_payload: Option<Json>,
    pub paid_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::apps::Entity",
        from = "Column::AppId",
        to = "super::apps::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    Apps,
    #[sea_orm(
        belongs_to = "super::license_plans::Entity",
        from = "Column::PlanId",
        to = "super::license_plans::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    LicensePlans,
    #[sea_orm(
        belongs_to = "super::reg_codes::Entity",
        from = "Column::RegCodeId",
        to = "super::reg_codes::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    RegCodes,
}

impl Related<super::apps::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Apps.def()
    }
}

impl Related<super::license_plans::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LicensePlans.def()
    }
}

impl Related<super::reg_codes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RegCodes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
