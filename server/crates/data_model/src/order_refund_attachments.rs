//! `SeaORM` Entity for protected order refund attachments.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "order_refund_attachments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub refund_id: i64,
    pub resource_id: i64,
    pub uploaded_by: i32,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::order_refunds::Entity",
        from = "Column::RefundId",
        to = "super::order_refunds::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    OrderRefunds,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UploadedBy",
        to = "super::users::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    Users,
    #[sea_orm(
        belongs_to = "super::resources::Entity",
        from = "Column::ResourceId",
        to = "super::resources::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    Resources,
}

impl Related<super::order_refunds::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OrderRefunds.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::resources::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Resources.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
