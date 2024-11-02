use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Posts::Table)
                    .add_column(small_integer(Posts::Category).default(1))
                    .add_column(string(Posts::TheAbstract).default(""))
                    .add_column(string(Posts::References).default(""))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Posts::Table)
                    .drop_column(Alias::new("category"))
                    .drop_column(Alias::new("the_abstract"))
                    .drop_column(Alias::new("references"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Posts {
    Table,
    Id,
    UserId,
    Title,
    SharingPath,
    Tags,
    Category,
    TheAbstract,
    Text,
    References,
    Status,
    CreatedAt,
    UpdatedAt,
}
