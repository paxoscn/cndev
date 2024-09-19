use ::entity::{post, post::Entity as Post};
use ::entity::{user, user::Entity as User};
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn find_post_by_id_and_user(db: &DbConn, id: i32, user_id: i32) -> Result<Option<post::Model>, DbErr> {
        Post::find()
        .filter(post::Column::Id.eq(id))
        .filter(post::Column::UserId.eq(user_id))
        .one(db).await
    }

    pub async fn find_post_by_id(db: &DbConn, id: i32) -> Result<Option<post::Model>, DbErr> {
        Post::find_by_id(id).one(db).await
    }

    /// If ok, returns (post models, num pages).
    pub async fn find_posts_in_page(
        db: &DbConn,
        page: u64,
        posts_per_page: u64,
    ) -> Result<(Vec<post::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = Post::find()
            .order_by_asc(post::Column::Id)
            .paginate(db, posts_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated posts
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }

    /// If ok, returns (post models, num pages).
    pub async fn find_posts_of_user_in_page(
        db: &DbConn,
        user_id: i32,
        page: u64,
        posts_per_page: u64,
    ) -> Result<(Vec<post::Model>, u64, u64), DbErr> {
        // Setup paginator
        let paginator = Post::find()
            .filter(post::Column::UserId.eq(user_id))
            .filter(post::Column::Status.ne(3))
            .order_by_desc(post::Column::Id)
            .paginate(db, posts_per_page);
        let num_pages = paginator.num_pages().await?;
        let total_count = paginator.num_items().await?;

        // Fetch paginated posts
        paginator.fetch_page(page - 1).await.map(|p| (p, total_count, num_pages))
    }

    pub async fn find_user_by_id(db: &DbConn, id: i32) -> Result<Option<user::Model>, DbErr> {
        User::find_by_id(id).one(db).await
    }

    /// If ok, returns (user models, num pages).
    pub async fn find_users_in_page(
        db: &DbConn,
        page: u64,
        users_per_page: u64,
    ) -> Result<(Vec<user::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = User::find()
            .order_by_asc(user::Column::Id)
            .paginate(db, users_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated users
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }

    pub async fn find_user_by_tel(db: &DbConn, tel: &str) -> Result<Option<user::Model>, DbErr> {
        User::find()
            .filter(user::Column::Tel.eq(tel))
            .one(db).await
    }
}
