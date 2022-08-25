//! SeaORM Entity. Generated by sea-orm-codegen 0.8.0

use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use sea_orm::InsertResult;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "course_review")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub course_id: i32,
    #[sea_orm(primary_key)]
    pub review_id: i32,
}

pub async fn link(course_id: i32, review_id: i32, db: &DatabaseConnection) -> Result<InsertResult<ActiveModel>, DbErr> {
    Ok(Entity::insert(ActiveModel {
        course_id: Set(course_id),
        review_id: Set(review_id),
    }).exec(db).await?)
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
    belongs_to = "super::course::Entity",
    from = "Column::CourseId",
    to = "super::course::Column::Id",
    on_update = "NoAction",
    on_delete = "Cascade"
    )]
    Course,
    #[sea_orm(
    belongs_to = "super::review::Entity",
    from = "Column::ReviewId",
    to = "super::review::Column::Id",
    on_update = "NoAction",
    on_delete = "Cascade"
    )]
    Review,
}


impl ActiveModelBehavior for ActiveModel {}