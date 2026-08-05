//! `SeaORM` Entity for protected settlement payment proofs.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "distribution_settlement_proofs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub settlement_id: i64,
    pub content: Vec<u8>,
    pub uploaded_by: i32,
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
        belongs_to = "super::users::Entity",
        from = "Column::UploadedBy",
        to = "super::users::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    Users,
}

impl Related<super::distribution_settlements::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DistributionSettlements.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
