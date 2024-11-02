use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

pub const STATUS_DRAFT: i16 = 1;
pub const STATUS_PUBLISHED: i16 = 2;
pub const STATUS_DELETED: i16 = 3;

pub const CATEGORY_ARTICLE: i16 = 1;
pub const CATEGORY_NOTE: i16 = 2;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub sharing_path: String,
    pub tags: String,
    pub category: i16,
    pub the_abstract: String,
    #[sea_orm(column_type = "Text")]
    pub text: String,
    pub references: String,
    pub status: i16,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    #[sea_orm(ignore)]
    pub updated_at_formatted: String,
    #[sea_orm(ignore)]
    pub old_sharing_path: String,
    #[sea_orm(ignore)]
    pub id_or_sharing_path: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
