use crate::sea_orm::{ConnectionTrait, TransactionTrait};
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220826_174342_create_table"
    }
}

fn course() -> TableCreateStatement {
    Table::create()
        .table(Alias::new("course"))
        .col(
            ColumnDef::new(Alias::new("id"))
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(Alias::new("name"))
                .custom(Alias::new("LONGTEXT"))
                .not_null(),
        )
        .col(
            ColumnDef::new(Alias::new("code"))
                .custom(Alias::new("LONGTEXT"))
                .not_null(),
        )
        .col(
            ColumnDef::new(Alias::new("code_id"))
                .custom(Alias::new("LONGTEXT"))
                .not_null(),
        )
        .col(ColumnDef::new(Alias::new("credit")).double().not_null())
        .col(
            ColumnDef::new(Alias::new("department"))
                .custom(Alias::new("LONGTEXT"))
                .not_null(),
        )
        .col(
            ColumnDef::new(Alias::new("campus_name"))
                .custom(Alias::new("LONGTEXT"))
                .not_null(),
        )
        .col(
            ColumnDef::new(Alias::new("teachers"))
                .custom(Alias::new("LONGTEXT"))
                .not_null(),
        )
        .col(
            ColumnDef::new(Alias::new("max_student"))
                .integer()
                .not_null(),
        )
        .col(ColumnDef::new(Alias::new("week_hour")).integer().not_null())
        .col(ColumnDef::new(Alias::new("year")).integer().not_null())
        .col(ColumnDef::new(Alias::new("semester")).integer().not_null())
        .to_owned()
}

fn course_review() -> TableCreateStatement {
    Table::create()
        .table(Alias::new("course_review"))
        .col(ColumnDef::new(Alias::new("course_id")).integer().not_null())
        .col(ColumnDef::new(Alias::new("review_id")).integer().not_null())
        .to_owned()
}

fn coursegroup() -> TableCreateStatement {
    Table::create()
        .table(Alias::new("coursegroup"))
        .col(
            ColumnDef::new(Alias::new("id"))
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(Alias::new("name"))
                .custom(Alias::new("LONGTEXT"))
                .not_null(),
        )
        .col(
            ColumnDef::new(Alias::new("code"))
                .custom(Alias::new("LONGTEXT"))
                .not_null(),
        )
        .col(
            ColumnDef::new(Alias::new("department"))
                .custom(Alias::new("LONGTEXT"))
                .not_null(),
        )
        .col(
            ColumnDef::new(Alias::new("campus_name"))
                .custom(Alias::new("LONGTEXT"))
                .not_null(),
        )
        .to_owned()
}

fn coursegroup_course() -> TableCreateStatement {
    Table::create()
        .table(Alias::new("coursegroup_course"))
        .col(
            ColumnDef::new(Alias::new("coursegroup_id"))
                .integer()
                .not_null(),
        )
        .col(ColumnDef::new(Alias::new("course_id")).integer().not_null())
        .to_owned()
}

fn review() -> TableCreateStatement {
    Table::create()
        .table(Alias::new("review"))
        .col(
            ColumnDef::new(Alias::new("id"))
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(Alias::new("title"))
                .custom(Alias::new("LONGTEXT"))
                .not_null(),
        )
        .col(
            ColumnDef::new(Alias::new("content"))
                .custom(Alias::new("LONGTEXT"))
                .not_null(),
        )
        .col(ColumnDef::new(Alias::new("history")).json().not_null())
        .col(
            ColumnDef::new(Alias::new("reviewer_id"))
                .integer()
                .not_null(),
        )
        .col(
            ColumnDef::new(Alias::new("time_created"))
                .date_time()
                .not_null(),
        )
        .col(
            ColumnDef::new(Alias::new("time_updated"))
                .date_time()
                .not_null(),
        )
        .col(ColumnDef::new(Alias::new("rank")).json().not_null())
        .col(ColumnDef::new(Alias::new("upvoters")).json().not_null())
        .col(ColumnDef::new(Alias::new("downvoters")).json().not_null())
        .to_owned()
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();
        let transaction = db.begin().await?;
        for mut table in [
            course(),
            course_review(),
            coursegroup(),
            coursegroup_course(),
            review(),
        ] {
            transaction
                .execute(backend.build(table.if_not_exists()))
                .await?;
        }
        transaction.commit().await
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        todo!()
    }
}
