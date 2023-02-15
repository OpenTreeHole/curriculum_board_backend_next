mod m20220826_174342_create_table;
mod m20230119_125830_create_user_extra;
mod m20230214_202755_related_to_foreign;
mod m20230215_111344_userextra_to_achievement;

pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220826_174342_create_table::Migration),
            Box::new(m20230119_125830_create_user_extra::Migration),
            Box::new(m20230214_202755_related_to_foreign::Migration),
            Box::new(m20230215_111344_userextra_to_achievement::Migration),
        ]
    }
}
