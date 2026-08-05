//! `SeaORM` Entity for adjustment offset allocations.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "distribution_commission_adjustment_offsets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub adjustment_id: i64,
    pub commission_id: i64,
    pub amount_cents: i32,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::distribution_commission_adjustments::Entity",
        from = "Column::AdjustmentId",
        to = "super::distribution_commission_adjustments::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    DistributionCommissionAdjustments,
    #[sea_orm(
        belongs_to = "super::distribution_commissions::Entity",
        from = "Column::CommissionId",
        to = "super::distribution_commissions::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    DistributionCommissions,
}

impl Related<super::distribution_commission_adjustments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DistributionCommissionAdjustments.def()
    }
}

impl Related<super::distribution_commissions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DistributionCommissions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
