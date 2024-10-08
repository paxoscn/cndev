
use cndev_service::{
    Mutation, Query,
};

use entity::{post::{self, STATUS_PUBLISHED}, user};
use migration::sea_orm::ColIdx;
use serde::{Serialize, Deserialize};
use crate::controllers::AppState;

use cndev_service::sea_orm::DatabaseConnection;

use std::env;

use crate::shencha;

use ftp::FtpStream;
use std::io::Cursor;

use cndev_service::sea_orm::TryIntoModel;

use actix_web::{
    error, get, post, put, delete, web, Error, HttpRequest, HttpResponse, Result, http::header::HeaderValue,
};

const DEFAULT_POSTS_PER_PAGE: u64 = 100;

#[derive(Debug, Deserialize)]
pub struct Params {
    status: i16,
    page: Option<u64>,
    posts_per_page: Option<u64>,
}

#[derive(Deserialize)]
struct PostSavingRequest {
    title: String,
    sharing_path: String,
    tags: String,
    text: String,
}


#[derive(Serialize)]
struct PostListingResponse<'a> {
    entities: &'a Vec<post::Model>,
    page: u64,
    entities_per_page: u64,
    num_pages: u64,
}

// curl -v -H "Authorization: Bearer $(cat /tmp/token)" 'http://127.0.0.1:8000/posts'
#[get("/posts")]
async fn list(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let user_id = match req.headers().get("id") {
        Some(id) => id.to_str().unwrap().parse::<i32>().unwrap(),
        None => {
            return Ok(HttpResponse::NotFound().finish())
        }
    };

    let conn = &data.conn;

    // get params
    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(DEFAULT_POSTS_PER_PAGE);

    let (mut posts, total_count, num_pages) = Query::find_posts_of_user_and_status_in_page(conn, user_id, params.status, page, posts_per_page)
        .await
        .expect("Cannot find posts in page");

    for post in posts.iter_mut() {
        post.updated_at_formatted = post.updated_at.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    Ok(HttpResponse::Ok().json(PostListingResponse {
        entities: &posts,
        page: page,
        entities_per_page: posts_per_page,
        num_pages: num_pages,
    }))
}

// curl -v -XPOST -H 'Content-Type: application/json' -H "Authorization: Bearer $(cat /tmp/token)" 'http://127.0.0.1:8000/posts' -d '{ "title": "t1", "tags": "t1,t2", "text": "tt1" }'
#[post("/posts")]
async fn create(
    req: HttpRequest,
    data: web::Data<AppState>,
    post_saving_request_json: web::Json<PostSavingRequest>,
) -> Result<HttpResponse, Error> {
    let user_id = match req.headers().get("id") {
        Some(id) => id.to_str().unwrap().parse::<i32>().unwrap(),
        None => {
            return Ok(HttpResponse::NotFound().finish())
        }
    };

    let post_saving_request = post_saving_request_json.into_inner();

    let conn = &data.conn;
    
    let saved_post = match Mutation::create_post(conn,
            user_id,
            post_saving_request.title,
            post_saving_request.sharing_path,
            post_saving_request.tags,
            post_saving_request.text).await {
        Ok(saved_post) => saved_post,
        Err(e) => {
            print!("Database error: {:?}", e);

            return Ok(HttpResponse::InternalServerError().finish())
        }
    };

    Ok(HttpResponse::Created().json(saved_post.try_into_model().unwrap()))
}

#[get("/posts/{id}")]
async fn load(
    req: HttpRequest,
    data: web::Data<AppState>,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let id = id.into_inner();

    let user_id = match req.headers().get("id") {
        Some(id) => id.to_str().unwrap().parse::<i32>().unwrap(),
        None => {
            return Ok(HttpResponse::NotFound().finish())
        }
    };

    let post: post::Model = Query::find_post_by_id_and_user(conn, id, user_id)
        .await
        .expect("could not find post")
        .unwrap_or_else(|| panic!("could not find post with id {id}"));

    Ok(HttpResponse::Ok().json(post))
}

#[put("/posts/{id}")]
async fn update(
    req: HttpRequest,
    data: web::Data<AppState>,
    id: web::Path<i32>,
    post_saving_request_json: web::Json<PostSavingRequest>,
) -> Result<HttpResponse, Error> {
    let user_id = match req.headers().get("id") {
        Some(id) => id.to_str().unwrap().parse::<i32>().unwrap(),
        None => {
            return Ok(HttpResponse::NotFound().finish())
        }
    };
    let user_nick = req.headers().get("nick").unwrap().to_str().unwrap();
    let user_registering_time = req.headers().get("reg").unwrap().to_str().unwrap().parse::<i64>().unwrap();

    let id = id.into_inner();
    let post_saving_request = post_saving_request_json.into_inner();

    match shencha_text(format!("{} {} {}", post_saving_request.title, post_saving_request.sharing_path, post_saving_request.tags).as_str()) {
        Ok(true) => {}
        Ok(false) => {
            return Ok(HttpResponse::Forbidden().finish())
        }
        Err(e) => {
            print!("Shencha error: {:?}", e);

            return Ok(HttpResponse::InternalServerError().finish())
        }
    }

    match shencha_text(post_saving_request.text.as_str()) {
        Ok(true) => {}
        Ok(false) => {
            return Ok(HttpResponse::Forbidden().finish())
        }
        Err(e) => {
            print!("Shencha error: {:?}", e);

            return Ok(HttpResponse::InternalServerError().finish())
        }
    }

    let conn = &data.conn;
    
    let mut saved_post = match Mutation::update_post_by_id(conn,
            user_id,
            id,
            post_saving_request.title,
            post_saving_request.sharing_path,
            post_saving_request.tags,
            post_saving_request.text).await {
        Ok(saved_post) => saved_post,
        Err(e) => {
            print!("Database error: {:?}", e);

            return Ok(HttpResponse::InternalServerError().finish())
        }
    };

    if saved_post.status == STATUS_PUBLISHED {
        let page = 1;
        let posts_per_page = DEFAULT_POSTS_PER_PAGE;
    
        let (posts, total_count, num_pages) = Query::find_posts_of_user_and_status_in_page(conn, user_id, STATUS_PUBLISHED, page, posts_per_page)
            .await
            .expect("Cannot find posts in page");
    
        let host = "127.0.0.1";
        let username = "root";
        let password = "root";
        
        // Bad vsftpd may hang here. Restart vsftpd to fix.
        let mut ftp = FtpStream::connect((host, 21)).unwrap();
        ftp.login(username, password).unwrap();
    
        publish_post_page(&mut ftp, &data, user_id, user_nick, user_registering_time, &mut saved_post).await;
    
        publish_home_page(&mut ftp, &data, user_id, user_nick, user_registering_time, posts, total_count, num_pages, page, posts_per_page).await;
    
        // Double-quitting leads panicking.
        ftp.quit().unwrap();
    }

    Ok(HttpResponse::Ok().json(saved_post.try_into_model().unwrap()))
}

#[put("/posts/{id}/publishing")]
async fn publish(
    req: HttpRequest,
    data: web::Data<AppState>,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let user_id = match req.headers().get("id") {
        Some(id) => id.to_str().unwrap().parse::<i32>().unwrap(),
        None => {
            return Ok(HttpResponse::NotFound().finish())
        }
    };
    let user_nick = req.headers().get("nick").unwrap().to_str().unwrap();
    let user_registering_time = req.headers().get("reg").unwrap().to_str().unwrap().parse::<i64>().unwrap();

    let id = id.into_inner();

    let conn = &data.conn;
    
    let mut saved_post = match Mutation::publish_post_by_id(conn,
            user_id,
            id).await {
        Ok(saved_post) => saved_post,
        Err(e) => {
            print!("Database error: {:?}", e);

            return Ok(HttpResponse::InternalServerError().finish())
        }
    };

    let page = 1;
    let posts_per_page = DEFAULT_POSTS_PER_PAGE;

    let (posts, total_count, num_pages) = Query::find_posts_of_user_and_status_in_page(conn, user_id, STATUS_PUBLISHED, page, posts_per_page)
        .await
        .expect("Cannot find posts in page");

    let host = "127.0.0.1";
    let username = "root";
    let password = "root";
    
    // Bad vsftpd may hang here. Restart vsftpd to fix.
    let mut ftp = FtpStream::connect((host, 21)).unwrap();
    ftp.login(username, password).unwrap();

    publish_post_page(&mut ftp, &data, user_id, user_nick, user_registering_time, &mut saved_post).await;

    publish_home_page(&mut ftp, &data, user_id, user_nick, user_registering_time, posts, total_count, num_pages, page, posts_per_page).await;

    // Double-quitting leads panicking.
    ftp.quit().unwrap();

    Ok(HttpResponse::Ok().finish())
}

#[put("/posts/{id}/unpublishing")]
async fn unpublish(
    req: HttpRequest,
    data: web::Data<AppState>,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let user_id = match req.headers().get("id") {
        Some(id) => id.to_str().unwrap().parse::<i32>().unwrap(),
        None => {
            return Ok(HttpResponse::NotFound().finish())
        }
    };
    let user_nick = req.headers().get("nick").unwrap().to_str().unwrap();
    let user_registering_time = req.headers().get("reg").unwrap().to_str().unwrap().parse::<i64>().unwrap();

    let id = id.into_inner();

    let conn = &data.conn;
    
    let mut saved_post = match Mutation::unpublish_post_by_id(conn,
            user_id,
            id).await {
        Ok(saved_post) => saved_post,
        Err(e) => {
            print!("Database error: {:?}", e);

            return Ok(HttpResponse::InternalServerError().finish())
        }
    };

    let page = 1;
    let posts_per_page = DEFAULT_POSTS_PER_PAGE;

    let (posts, total_count, num_pages) = Query::find_posts_of_user_and_status_in_page(conn, user_id, STATUS_PUBLISHED, page, posts_per_page)
        .await
        .expect("Cannot find posts in page");

    let host = "127.0.0.1";
    let username = "root";
    let password = "root";
    
    // Bad vsftpd may hang here. Restart vsftpd to fix.
    let mut ftp = FtpStream::connect((host, 21)).unwrap();
    ftp.login(username, password).unwrap();

    unpublish_post_page(&mut ftp, &data, user_id, user_nick, user_registering_time, &mut saved_post).await;

    publish_home_page(&mut ftp, &data, user_id, user_nick, user_registering_time, posts, total_count, num_pages, page, posts_per_page).await;

    // Double-quitting leads panicking.
    ftp.quit().unwrap();

    Ok(HttpResponse::Ok().finish())
}

// #[get("/{id}")]
// async fn edit(data: web::Data<AppState>, id: web::Path<i32>) -> Result<HttpResponse, Error> {
//     let conn = &data.conn;
//     let template = &data.templates;
//     let id = id.into_inner();

//     let post: post::Model = Query::find_post_by_id(conn, id)
//         .await
//         .expect("could not find post")
//         .unwrap_or_else(|| panic!("could not find post with id {id}"));

//     let mut ctx = tera::Context::new();
//     ctx.insert("post", &post);

//     let body = template
//         .render("edit.html.tera", &ctx)
//         .map_err(|_| error::ErrorInternalServerError("Template error"))?;
//     Ok(HttpResponse::Ok().content_type("text/html").body(body))
// }

// #[post("/{id}")]
// async fn update(
//     data: web::Data<AppState>,
//     id: web::Path<i32>,
//     post_form: web::Form<post::Model>,
// ) -> Result<HttpResponse, Error> {
//     let conn = &data.conn;
//     let form = post_form.into_inner();
//     let id = id.into_inner();

//     Mutation::update_post_by_id(conn, id, form)
//         .await
//         .expect("could not edit post");

//     Ok(HttpResponse::Found()
//         .append_header(("location", "/"))
//         .finish())
// }

#[delete("/posts/{id}")]
async fn delete(
    req: HttpRequest,
    data: web::Data<AppState>,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let user_id = match req.headers().get("id") {
        Some(id) => id.to_str().unwrap().parse::<i32>().unwrap(),
        None => {
            return Ok(HttpResponse::NotFound().finish())
        }
    };
    let user_nick = req.headers().get("nick").unwrap().to_str().unwrap();
    let user_registering_time = req.headers().get("reg").unwrap().to_str().unwrap().parse::<i64>().unwrap();

    let conn = &data.conn;
    let id = id.into_inner();
    
    let mut saved_post = match Mutation::delete_post_by_id(conn,
            user_id,
            id).await {
        Ok(saved_post) => saved_post,
        Err(e) => {
            print!("Database error: {:?}", e);

            return Ok(HttpResponse::InternalServerError().finish())
        }
    };

    let page = 1;
    let posts_per_page = DEFAULT_POSTS_PER_PAGE;

    let (posts, total_count, num_pages) = Query::find_posts_of_user_and_status_in_page(conn, user_id, STATUS_PUBLISHED, page, posts_per_page)
        .await
        .expect("Cannot find posts in page");

    let host = "127.0.0.1";
    let username = "root";
    let password = "root";
    
    // Bad vsftpd may hang here. Restart vsftpd to fix.
    let mut ftp = FtpStream::connect((host, 21)).unwrap();
    ftp.login(username, password).unwrap();

    unpublish_post_page(&mut ftp, &data, user_id, user_nick, user_registering_time, &mut saved_post).await;
    // publish_post_page(&mut ftp, &data, user_id, user_nick, user_registering_time, &mut saved_post).await;

    publish_home_page(&mut ftp, &data, user_id, user_nick, user_registering_time, posts, total_count, num_pages, page, posts_per_page).await;

    // Double-quitting leads panicking.
    ftp.quit().unwrap();

    Ok(HttpResponse::Ok().finish())
}

async fn publish_post_page(ftp: &mut FtpStream, data: &web::Data<AppState>, author_id: i32, author_nick: &str, author_registering_time: i64,
        post: &mut post::Model) -> bool {
    print!("Publishing post page {} for user {}", post.id, author_id);

    post.updated_at_formatted = post.updated_at.format("%Y-%m-%d %H:%M:%S").to_string();

    let template = &data.templates;
    let mut ctx = tera::Context::new();
    ctx.insert("template", "post");
    ctx.insert("author_id", &author_id);
    ctx.insert("author_nick", author_nick);
    ctx.insert("author_registering_time", &format!("{}", author_registering_time));
    ctx.insert("post", post);

    let body = template
        .render("post.html.tera", &ctx)
        .unwrap()
        .into_bytes();

    let _ = ftp.mkdir(format!("/{}", author_id).as_str());
    let _ = ftp.cwd(format!("/{}", author_id).as_str());
    let _ = ftp.put(format!("{}.html", post.id).as_str(), &mut Cursor::new(&body));
    if post.old_sharing_path.len() > 0 && post.old_sharing_path != post.sharing_path {
        let _ = ftp.rm(format!("{}.html", post.old_sharing_path).as_str());
    }
    if post.sharing_path.len() > 0 {
        let _ = ftp.put(format!("{}.html", post.sharing_path).as_str(), &mut Cursor::new(&body));
    }
    if author_nick.len() > 0 {
        let _ = ftp.mkdir(format!("/{}", author_nick).as_str());
        let _ = ftp.cwd(format!("/{}", author_nick).as_str());
    } else {
        let _ = ftp.mkdir(format!("/{}_", author_id).as_str());
        let _ = ftp.cwd(format!("/{}_", author_id).as_str());
    }
    let _ = ftp.put(format!("{}.html", post.id).as_str(), &mut Cursor::new(&body));
    if post.old_sharing_path.len() > 0 && post.old_sharing_path != post.sharing_path {
        let _ = ftp.rm(format!("{}.html", post.old_sharing_path).as_str());
    }
    if post.sharing_path.len() > 0 {
        let _ = ftp.put(format!("{}.html", post.sharing_path).as_str(), &mut Cursor::new(&body));
    }

    print!("Published post page {} for user {}", post.id, author_id);

    false
}

async fn unpublish_post_page(ftp: &mut FtpStream, data: &web::Data<AppState>, author_id: i32, author_nick: &str, author_registering_time: i64,
        post: &mut post::Model) -> bool {
    print!("Unpublishing post page {} for user {}", post.id, author_id);

    let _ = ftp.cwd(format!("/{}", author_id).as_str());
    let _ = ftp.rm(format!("{}.html", post.id).as_str());
    if post.sharing_path.len() > 0 {
        let _ = ftp.rm(format!("{}.html", post.sharing_path).as_str());
    }
    if author_nick.len() > 0 {
        let _ = ftp.cwd(format!("/{}", author_nick).as_str());
    } else {
        let _ = ftp.cwd(format!("/{}_", author_id).as_str());
    }
    let _ = ftp.rm(format!("{}.html", post.id).as_str());
    if post.sharing_path.len() > 0 {
        let _ = ftp.rm(format!("{}.html", post.sharing_path).as_str());
    }

    print!("Unpublished post page {} for user {}", post.id, author_id);

    false
}

pub async fn publish_home_page(ftp: &mut FtpStream, data: &web::Data<AppState>, author_id: i32, author_nick: &str, author_registering_time: i64,
        mut posts: Vec<post::Model>, total_count: u64, num_pages: u64, page: u64, posts_per_page: u64) -> bool {
    print!("Publishing home page for user {}", author_id);

    for post in posts.iter_mut() {
        post.updated_at_formatted = post.updated_at.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    let template = &data.templates;
    let mut ctx: tera::Context = tera::Context::new();
    ctx.insert("template", "home");
    ctx.insert("author_id", &author_id);
    ctx.insert("author_nick", author_nick);
    ctx.insert("author_registering_time", format!("{}", author_registering_time).as_str());
    ctx.insert("posts", &posts);
    ctx.insert("page", &page);
    ctx.insert("posts_per_page", &posts_per_page);
    ctx.insert("num_pages", &num_pages);
    ctx.insert("total_count", &total_count);

    let body = template
        .render("home.html.tera", &ctx)
        .unwrap()
        .into_bytes();

    let _ = ftp.cwd("/");
    let _ = ftp.put(format!("{}.html", author_id).as_str(), &mut Cursor::new(&body));
    if author_nick.len() > 0 {
        let _ = ftp.put(format!("{}.html", author_nick).as_str(), &mut Cursor::new(&body));
    } else {
        let _ = ftp.put(format!("{}_.html", author_id).as_str(), &mut Cursor::new(&body));
    }

    print!("Published home page of user {}", author_id);

    false
}

fn shencha_text(text: &str) -> Result<bool, std::io::Error> {
    // TODO Init on startup.
    let aliyun_shencha_region = env::var("ALIYUN_SHENCHA_REGION").expect("ALIYUN_SHENCHA_REGION is not set in .env file");
    let aliyun_shencha_ak = env::var("ALIYUN_SHENCHA_AK").expect("ALIYUN_SHENCHA_AK is not set in .env file");
    let aliyun_shencha_sk = env::var("ALIYUN_SHENCHA_SK").expect("ALIYUN_SHENCHA_SK is not set");

    let mut aliyun_client = alibaba_cloud_sdk_rust::services::dysmsapi::Client::NewClientWithAccessKey(
        aliyun_shencha_region.as_str(),
        aliyun_shencha_ak.as_str(),
        aliyun_shencha_sk.as_str(),
    )?;

    let mut chunks = Vec::new();
    let mut chars = text.chars();
    let mut current_chunk = String::new();

    while let Some(c) = chars.next() {
        current_chunk.push(c);
        if current_chunk.len() >= 500 {
            chunks.push(current_chunk);
            current_chunk = String::new();
        }
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    for chunk in chunks {
        match shencha::shencha(&mut aliyun_client, "comment_detection", &chunk) {
            Ok(true) => {},
            Ok(false) => return Ok(false),
            Err(e) => {
                println!("Error: {}", e);

                return Ok(false)
            },
        }
    }

    Ok(true)
}