//! `SeaORM` entity for email verification challenges.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "email_verification_challenges")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub email: String,
    pub purpose: String,
    #[serde(skip_serializing)]
    pub code_hash: String,
    pub attempts: i32,
    pub expires_at: DateTimeWithTimeZone,
    pub resend_after: DateTimeWithTimeZone,
    pub sent_at: Option<DateTimeWithTimeZone>,
    pub send_failed_at: Option<DateTimeWithTimeZone>,
    pub verified_at: Option<DateTimeWithTimeZone>,
    pub consumed_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::email_verification_tokens::Entity")]
    EmailVerificationTokens,
}

impl Related<super::email_verification_tokens::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EmailVerificationTokens.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
