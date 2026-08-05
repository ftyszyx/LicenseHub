//! `SeaORM` Entity for distribution withdrawal settlements.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "distribution_settlements")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub settlement_no: String,
    pub user_id: i32,
    pub amount_cents: i32,
    pub status: i16,
    pub settlement_account: Json,
    pub payment_reference: Option<String>,
    pub payment_proof_file_name: Option<String>,
    pub payment_proof_content_type: Option<String>,
    pub payment_proof_size: Option<i64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub reject_reason: Option<String>,
    pub requested_at: DateTimeWithTimeZone,
    pub reviewed_at: Option<DateTimeWithTimeZone>,
    pub paid_at: Option<DateTimeWithTimeZone>,
    pub reviewed_by: Option<i32>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::distribution_settlement_items::Entity")]
    DistributionSettlementItems,
    #[sea_orm(has_one = "super::distribution_settlement_proofs::Entity")]
    DistributionSettlementProofs,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    Users,
}

impl Related<super::distribution_settlement_items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DistributionSettlementItems.def()
    }
}

impl Related<super::distribution_settlement_proofs::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DistributionSettlementProofs.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
