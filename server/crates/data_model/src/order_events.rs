//! `SeaORM` Entity for order outbox events.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "order_events")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub order_id: i32,
    pub order_no: String,
    pub event_type: String,
    pub status: i16,
    pub payload: Option<Json>,
    pub processed_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::orders::Entity",
        from = "Column::OrderId",
        to = "super::orders::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Orders,
}

impl Related<super::orders::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Orders.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
