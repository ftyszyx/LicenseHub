//! `SeaORM` Entity for distribution commission records.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "distribution_commissions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub order_id: i32,
    pub user_id: i32,
    pub order_amount_cents: i32,
    pub commission_rate_bps: i32,
    pub commission_amount_cents: i32,
    pub status: i16,
    pub available_at: Option<DateTimeWithTimeZone>,
    pub locked_amount_cents: i32,
    pub settled_amount_cents: i32,
    pub cancelled_amount_cents: i32,
    pub adjustment_amount_cents: i32,
    #[sea_orm(column_type = "Text", nullable)]
    pub cancel_reason: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::distribution_commission_adjustment_offsets::Entity")]
    DistributionCommissionAdjustmentOffsets,
    #[sea_orm(has_many = "super::distribution_commission_adjustments::Entity")]
    DistributionCommissionAdjustments,
    #[sea_orm(has_many = "super::distribution_settlement_items::Entity")]
    DistributionSettlementItems,
    #[sea_orm(
        belongs_to = "super::orders::Entity",
        from = "Column::OrderId",
        to = "super::orders::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    Orders,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    Users,
}

impl Related<super::distribution_commission_adjustment_offsets::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DistributionCommissionAdjustmentOffsets.def()
    }
}

impl Related<super::distribution_commission_adjustments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DistributionCommissionAdjustments.def()
    }
}

impl Related<super::distribution_settlement_items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DistributionSettlementItems.def()
    }
}

impl Related<super::orders::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Orders.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
