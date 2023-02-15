use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20230119_125830_create_user_extra"
    }
}

fn userextra() -> TableCreateStatement {
    Table::create()
        .table(Alias::new("userextra"))
        .col(
            ColumnDef::new(Alias::new("user_id"))
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(Alias::new("extra"))
                .custom(Alias::new("LONGTEXT"))
                .not_null(),
        )
        .to_owned()
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(userextra().if_not_exists().to_owned())
            .await
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        todo!()
    }
}
