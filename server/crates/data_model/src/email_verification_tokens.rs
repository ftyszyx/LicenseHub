//! `SeaORM` entity for one-time email verification tokens.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "email_verification_tokens")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(skip_serializing)]
    pub token_hash: String,
    pub challenge_id: Uuid,
    pub email: String,
    pub purpose: String,
    pub expires_at: DateTimeWithTimeZone,
    pub consumed_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::email_verification_challenges::Entity",
        from = "Column::ChallengeId",
        to = "super::email_verification_challenges::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    EmailVerificationChallenges,
}

impl Related<super::email_verification_challenges::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EmailVerificationChallenges.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
