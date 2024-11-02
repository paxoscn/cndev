pub use sea_orm_migration::prelude::*;

mod m20240801_000001_create_user_table;
mod m20240802_000001_create_post_table;
mod m20241101_000001_alter_post_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240801_000001_create_user_table::Migration),
            Box::new(m20240802_000001_create_post_table::Migration),
            Box::new(m20241101_000001_alter_post_table::Migration)
        ]
    }
}
