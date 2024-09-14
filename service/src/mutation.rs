use ::entity::{post, post::Entity as Post};
use ::entity::{user, user::Entity as User};
use sea_orm::*;

use chrono::{DateTime, Utc};

pub struct Mutation;

impl Mutation {
    pub async fn create_post(
        db: &DbConn,
        user_id: i32,
        title: String,
        sharing_path: String,
        tags: String,
        text: String,
    ) -> Result<post::ActiveModel, DbErr> {
        post::ActiveModel {
            user_id: Set(user_id),
            title: Set(title.to_owned()),
            sharing_path: Set(sharing_path.to_owned()),
            tags: Set(tags.to_owned()),
            text: Set(text.to_owned()),
            status: Set(1),
            ..Default::default()
        }
        .save(db)
        .await
    }

    pub async fn update_post_by_id(
        db: &DbConn,
        user_id: i32,
        id: i32,
        title: String,
        sharing_path: String,
        tags: String,
        text: String,
    ) -> Result<post::Model, DbErr> {
        let post: post::ActiveModel = Post::find()
            .filter(post::Column::Id.eq(id))
            .filter(post::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
            .map(Into::into)?;

        println!("xxx {}", title.to_owned());

        post::ActiveModel {
            id: post.id,
            user_id: post.user_id,
            title: Set(title.to_owned()),
            sharing_path: Set(sharing_path.to_owned()),
            tags: Set(tags.to_owned()),
            text: Set(text.to_owned()),
            status: Set(post.status.as_ref().to_owned()),
            created_at: Set(post.created_at.as_ref().to_owned()),
            updated_at: Set(Utc::now().naive_utc().to_owned()),
        }
        .update(db)
        .await
    }

    pub async fn delete_post(db: &DbConn, id: i32) -> Result<DeleteResult, DbErr> {
        let post: post::ActiveModel = Post::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
            .map(Into::into)?;

        post.delete(db).await
    }

    pub async fn delete_all_posts(db: &DbConn) -> Result<DeleteResult, DbErr> {
        Post::delete_many().exec(db).await
    }

    pub async fn create_user(
        db: &DbConn,
        tel: &str,
    ) -> Result<user::ActiveModel, DbErr> {
        user::ActiveModel {
            nick: Set("".to_owned()),
            name: Set("".to_owned()),
            tel: Set(tel.to_owned()),
            mail: Set("".to_owned()),
            status: Set(1),
            ..Default::default()
        }
        .save(db)
        .await
    }

    pub async fn update_user_by_id(
        db: &DbConn,
        id: i32,
        form_data: user::Model,
    ) -> Result<user::Model, DbErr> {
        let user: user::ActiveModel = User::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find user.".to_owned()))
            .map(Into::into)?;

        user::ActiveModel {
            id: user.id,
            nick: Set(form_data.name.to_owned()),
            name: Set(form_data.name.to_owned()),
            tel: Set(form_data.name.to_owned()),
            mail: Set(form_data.name.to_owned()),
            status: Set(form_data.status.to_owned()),
            created_at: Set(form_data.created_at.to_owned()),
            updated_at: Set(form_data.updated_at.to_owned()),
        }
        .update(db)
        .await
    }

    pub async fn delete_user(db: &DbConn, id: i32) -> Result<DeleteResult, DbErr> {
        let user: user::ActiveModel = User::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find user.".to_owned()))
            .map(Into::into)?;

        user.delete(db).await
    }

    pub async fn delete_all_users(db: &DbConn) -> Result<DeleteResult, DbErr> {
        User::delete_many().exec(db).await
    }
}