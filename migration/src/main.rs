use sea_orm_migration::prelude::*;

fn main() {
    async_std::task::block_on(async {
        cli::run_cli(migration::Migrator).await;
    });
}
