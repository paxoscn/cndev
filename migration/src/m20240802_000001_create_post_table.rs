use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Posts::Table)
                    .if_not_exists()
                    .col(pk_auto(Posts::Id))
                    .col(integer(Posts::UserId))
                    .col(string(Posts::Title))
                    .col(string(Posts::SharingPath))
                    .col(string(Posts::Tags))
                    .col(string(Posts::Text))
                    .col(small_integer(Posts::Status))
                    .col(date_time(Posts::CreatedAt).default("now()".to_string()))
                    .col(date_time(Posts::UpdatedAt).default("now()".to_string()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Posts::Table).to_owned())
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
    Text,
    Status,
    CreatedAt,
    UpdatedAt,
}
