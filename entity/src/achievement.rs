use sea_orm::entity::prelude::*;
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};

use crate::user_achievement;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, DeriveEntityModel, ToSchema)]
#[sea_orm(table_name = "achievement")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "custom(\"LONGTEXT\")")]
    pub name: String,
    #[sea_orm(column_type = "custom(\"LONGTEXT\")")]
    pub domain: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user_achievement::Entity")]
    UserAchievement,
}

impl Related<user_achievement::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserAchievement.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

