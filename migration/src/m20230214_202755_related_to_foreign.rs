use crate::sea_orm::{ConnectionTrait, DatabaseTransaction, DbBackend, TransactionTrait};
use sea_orm_migration::prelude::*;
use std::borrow::Borrow;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20230214_202755_related_to_foreign"
    }
}

fn from_related_to_foreign_key(
    child_table: &str,
    related_table: &str,
    child_key_col: &str,
    child_foreign_col: &str,
    related_child_key_col: &str,
    related_parent_key_col: &str,
) -> String {
    format!(
        "UPDATE {child_table}
    LEFT JOIN {related_table} AS cc ON {child_table}.{child_key_col} = cc.{related_child_key_col}
    SET {child_table}.{child_foreign_col} = cc.{related_parent_key_col}"
    )
}

struct ForeignKeyMigration {
    child_tbl: String,
    related_tbl: String,
    parent_tbl: String,

    child_key_col: String,
    parent_key_col: String,

    child_foreign_col: String,

    related_child_key_col: String,
    related_parent_key_col: String,
}

impl ForeignKeyMigration {
    async fn migrate<T: ConnectionTrait>(
        &self,
        backend: impl Borrow<DbBackend>,
        conn: impl Borrow<T>,
    ) -> Result<(), DbErr> {
        let conn = conn.borrow();
        let backend = backend.borrow();
        // 在子表中添加外键列
        let sql = Table::alter()
            .table(Alias::new(self.child_tbl.as_str()))
            .add_column(
                ColumnDef::new(Alias::new(self.child_foreign_col.as_str()))
                    .integer()
                    .default(Value::Int(None)),
            )
            .add_foreign_key(
                TableForeignKey::new()
                    .from_tbl(Alias::new(self.child_tbl.as_str()))
                    .from_col(Alias::new(self.child_foreign_col.as_str()))
                    .to_tbl(Alias::new(self.parent_tbl.as_str()))
                    .to_col(Alias::new(self.parent_key_col.as_str()))
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .to_owned();

        conn.execute(backend.build(&sql)).await?;

        // 连接子表和关联表，将子表的外键列填充
        conn.execute_unprepared(
            from_related_to_foreign_key(
                self.child_tbl.as_str(),
                self.related_tbl.as_str(),
                self.child_key_col.as_str(),
                self.child_foreign_col.as_str(),
                self.related_child_key_col.as_str(),
                self.related_parent_key_col.as_str(),
            )
                .as_str(),
        )
            .await?;

        // 删除关联表
        let sql = Table::drop()
            .table(Alias::new(self.related_tbl.as_str()))
            .to_owned();
        conn.execute(backend.build(&sql)).await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let transaction = db.begin().await?;
        let backend = manager.get_database_backend();

        let course_group_course = ForeignKeyMigration {
            child_tbl: "course".to_string(),
            related_tbl: "coursegroup_course".to_string(),
            parent_tbl: "coursegroup".to_string(),
            child_key_col: "id".to_string(),
            parent_key_col: "id".to_string(),
            child_foreign_col: "coursegroup_id".to_string(),
            related_child_key_col: "course_id".to_string(),
            related_parent_key_col: "coursegroup_id".to_string(),
        };
        course_group_course
            .migrate::<DatabaseTransaction>(&backend, &transaction)
            .await?;

        let course_review = ForeignKeyMigration {
            child_tbl: "review".to_string(),
            related_tbl: "course_review".to_string(),
            parent_tbl: "course".to_string(),
            child_key_col: "id".to_string(),
            parent_key_col: "id".to_string(),
            child_foreign_col: "course_id".to_string(),
            related_child_key_col: "review_id".to_string(),
            related_parent_key_col: "course_id".to_string(),
        };
        course_review
            .migrate::<DatabaseTransaction>(&backend, &transaction)
            .await?;

        transaction.commit().await
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        todo!()
    }
}
