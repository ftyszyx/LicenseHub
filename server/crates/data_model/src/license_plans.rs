//! `SeaORM` Entity for license plans.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "license_plans")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub app_id: i32,
    pub name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub price_cents: i32,
    pub code_type: i16,
    pub valid_days: i32,
    pub total_count: Option<i32>,
    pub status: i16,
    pub sort_order: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::apps::Entity",
        from = "Column::AppId",
        to = "super::apps::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Apps,
    #[sea_orm(has_many = "super::orders::Entity")]
    Orders,
}

impl Related<super::apps::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Apps.def()
    }
}

impl Related<super::orders::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Orders.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
