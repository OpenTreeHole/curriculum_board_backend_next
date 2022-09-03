use sea_orm_migration::prelude::*;
use entity::prelude::*;
use crate::sea_orm::Schema;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220826_174342_create_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(manager.get_database_backend());
        manager.create_table(schema.create_table_from_entity(Course).if_not_exists().to_owned()).await?;
        manager.create_table(schema.create_table_from_entity(CourseReview).if_not_exists().to_owned()).await?;
        manager.create_table(schema.create_table_from_entity(Coursegroup).if_not_exists().to_owned()).await?;
        manager.create_table(schema.create_table_from_entity(CoursegroupCourse).if_not_exists().to_owned()).await?;
        manager.create_table(schema.create_table_from_entity(Review).if_not_exists().to_owned()).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        todo!()
    }
}
