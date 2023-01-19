mod m20220826_174342_create_table;
mod m20230119_125830_create_user_extra;


pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220826_174342_create_table::Migration),
            Box::new(m20230119_125830_create_user_extra::Migration),
        ]
    }
}
