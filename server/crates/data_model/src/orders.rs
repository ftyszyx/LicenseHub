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
    pub buyer_user_id: Option<i32>,
    pub buyer_email: Option<String>,
    pub referrer_user_id: Option<i32>,
    pub referral_code: Option<String>,
    pub commission_rate_bps: Option<i32>,
    pub commission_amount_cents: Option<i32>,
    pub paid_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::distribution_commissions::Entity")]
    DistributionCommissions,
    #[sea_orm(has_one = "super::order_refunds::Entity")]
    OrderRefunds,
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
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::ReferrerUserId",
        to = "super::users::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    ReferrerUser,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::BuyerUserId",
        to = "super::users::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    BuyerUser,
}

impl Related<super::distribution_commissions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DistributionCommissions.def()
    }
}

impl Related<super::order_refunds::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OrderRefunds.def()
    }
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

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BuyerUser.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
