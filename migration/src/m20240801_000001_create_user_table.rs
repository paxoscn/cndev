use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(pk_auto(Users::Id))
                    .col(string_null(Users::Nick).unique_key())
                    .col(string(Users::Name))
                    .col(string_uniq(Users::Tel))
                    .col(string(Users::Mail))
                    .col(small_integer(Users::Status))
                    .col(date_time(Users::CreatedAt).default("now()".to_string()))
                    .col(date_time(Users::UpdatedAt).default("now()".to_string()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Nick,
    Name,
    Tel,
    Mail,
    Status,
    CreatedAt,
    UpdatedAt,
}
