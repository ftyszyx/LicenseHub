//! `SeaORM` Entity for commission allocations in a settlement.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "distribution_settlement_items")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub settlement_id: i64,
    pub commission_id: i64,
    pub amount_cents: i32,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::distribution_settlements::Entity",
        from = "Column::SettlementId",
        to = "super::distribution_settlements::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    DistributionSettlements,
    #[sea_orm(
        belongs_to = "super::distribution_commissions::Entity",
        from = "Column::CommissionId",
        to = "super::distribution_commissions::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    DistributionCommissions,
}

impl Related<super::distribution_settlements::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DistributionSettlements.def()
    }
}

impl Related<super::distribution_commissions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DistributionCommissions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
