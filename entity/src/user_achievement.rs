use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::achievement;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, DeriveEntityModel, ToSchema)]
#[sea_orm(table_name = "user_achievement")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_id: i32,
    #[sea_orm(primary_key)]
    pub achievement_id: i32,
    pub obtain_date: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
    belongs_to = "super::achievement::Entity",
    from = "Column::AchievementId",
    to = "super::achievement::Column::Id"
    )]
    Achievement,
}

impl Related<achievement::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Achievement.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl GetAchievement {
    pub async fn load(user_id: i32, db: &DatabaseConnection) -> Result<Vec<Self>, DbErr> {
        let models = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .all(db)
            .await?;
        let aches = models.load_one(achievement::Entity, db).await?;

        Ok(models
            .into_iter()
            .zip(aches)
            .filter_map(|(model, ache)| {
                ache.map(|ache| GetAchievement {
                    name: ache.name,
                    domain: ache.domain,
                    obtain_date: model.obtain_date,
                })
            })
            .collect::<Vec<_>>())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct GetAchievement {
    name: String,
    domain: Option<String>,
    obtain_date: DateTime,
}
