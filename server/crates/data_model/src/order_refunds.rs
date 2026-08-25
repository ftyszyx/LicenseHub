//! `SeaORM` Entity for confirmed order refund records.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "order_refunds")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub refund_no: String,
    #[sea_orm(unique)]
    pub order_id: i32,
    pub amount_cents: i32,
    pub provider: String,
    pub provider_trade_no: Option<String>,
    pub refund_reference: String,
    pub reason: String,
    pub status: i16,
    pub operator_user_id: i32,
    pub refunded_at: DateTimeWithTimeZone,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
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
        from = "Column::OperatorUserId",
        to = "super::users::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    Users,
    #[sea_orm(has_one = "super::order_refund_attachments::Entity")]
    OrderRefundAttachments,
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

impl Related<super::order_refund_attachments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OrderRefundAttachments.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
