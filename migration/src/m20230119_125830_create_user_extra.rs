use sea_orm_migration::prelude::*;
use entity::prelude::*;
use entity::userextra;
use crate::sea_orm::Schema;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20230119_125830_create_user_extra"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(manager.get_database_backend());
        Ok(manager.create_table(schema.create_table_from_entity(Userextra).if_not_exists().to_owned()).await?)
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        todo!()
    }
}
