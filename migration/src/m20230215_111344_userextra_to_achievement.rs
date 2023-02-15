use crate::sea_orm::{ConnectionTrait, TransactionTrait};
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20230215_111344_userextra_to_achievement"
    }
}

fn achievement() -> TableCreateStatement {
    Table::create()
        .table(Alias::new("achievement"))
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
        .col(ColumnDef::new(Alias::new("domain")).custom(Alias::new("LONGTEXT")))
        .to_owned()
}

fn user_achievement() -> TableCreateStatement {
    Table::create()
        .table(Alias::new("user_achievement"))
        .col(ColumnDef::new(Alias::new("user_id")).integer().not_null())
        .col(
            ColumnDef::new(Alias::new("achievement_id"))
                .integer()
                .not_null(),
        )
        .col(
            ColumnDef::new(Alias::new("obtain_date"))
                .date_time()
                .not_null(),
        )
        .to_owned()
}

const GENERATE_ACHIEVEMENTS: &str = r#"
INSERT INTO achievement ( name, domain ) 
SELECT DISTINCT ach_name, ach_domain 
FROM userextra
CROSS JOIN JSON_TABLE (
	userextra.extra,
	'$.achievements[*]' COLUMNS ( ach_name LONGTEXT PATH '$.name' ERROR ON ERROR, ach_domain LONGTEXT PATH '$.domain' ERROR ON ERROR )) AS jt
"#;

const MIGRATE_USER_ACHIEVEMENTS: &str = r#"
INSERT INTO user_achievement ( user_id, achievement_id, obtain_date )
SELECT
	userextra.user_id,
	(
	SELECT
		id 
	FROM
		achievement 
	WHERE
        BINARY achievement.`name` = BINARY ach_name 
		AND (
			BINARY achievement.domain = BINARY ach_domain 
		OR ( achievement.domain IS NULL AND ach_domain IS NULL ))),
	CAST( ach_date AS datetime ) 
FROM
	userextra
CROSS JOIN JSON_TABLE (
	userextra.extra,
	'$.achievements[*]' COLUMNS ( ach_name LONGTEXT PATH '$.name' ERROR ON ERROR, ach_domain LONGTEXT PATH '$.domain' ERROR ON ERROR, ach_date LONGTEXT PATH '$.obtain_date' ERROR ON ERROR )) AS jt
"#;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();
        let transaction = db.begin().await?;

        // 创建新表
        transaction
            .execute(backend.build(achievement().if_not_exists()))
            .await?;

        transaction
            .execute(backend.build(user_achievement().if_not_exists()))
            .await?;

        // 向 acheivement 表中插入所有已有的成就
        transaction
            .execute_unprepared(GENERATE_ACHIEVEMENTS)
            .await?;

        // 将 userextra 表中的成就信息转移到 user_achievement 表中
        transaction
            .execute_unprepared(MIGRATE_USER_ACHIEVEMENTS)
            .await?;

        transaction.commit().await
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        todo!()
    }
}
