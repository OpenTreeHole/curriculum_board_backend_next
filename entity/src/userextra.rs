use sea_orm::entity::prelude::*;
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, DeriveEntityModel, ToSchema)]
#[sea_orm(table_name = "userextra")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_id: i32,
    #[sea_orm(column_type = "Custom(\"LONGTEXT\".to_owned())")]
    pub extra: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}