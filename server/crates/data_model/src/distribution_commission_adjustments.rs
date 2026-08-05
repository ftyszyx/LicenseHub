//! `SeaORM` Entity for negative commission adjustments.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "distribution_commission_adjustments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: i32,
    pub order_id: i32,
    pub original_commission_id: i64,
    pub amount_cents: i32,
    pub offset_amount_cents: i32,
    pub reason: String,
    pub status: i16,
    pub operator_user_id: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::distribution_commission_adjustment_offsets::Entity")]
    DistributionCommissionAdjustmentOffsets,
    #[sea_orm(
        belongs_to = "super::distribution_commissions::Entity",
        from = "Column::OriginalCommissionId",
        to = "super::distribution_commissions::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    DistributionCommissions,
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

impl Related<super::distribution_commissions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DistributionCommissions.def()
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
