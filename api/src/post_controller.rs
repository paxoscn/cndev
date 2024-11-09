
use cndev_service::{
    Mutation, Query,
};

use entity::post::{self, STATUS_PUBLISHED};
use serde::{Serialize, Deserialize};
use crate::controllers::AppState;

use std::env;

use crate::shencha;

use cndev_service::sea_orm::TryIntoModel;

use actix_web::{
    get, post, put, delete, web, Error, HttpRequest, HttpResponse, Result,
};

use actix_multipart::form::{tempfile::TempFile, MultipartForm};

use chrono::Utc;

use std::io::Read;
use std::io::Write;

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
    category: i16,
    the_abstract: String,
    text: String,
    references: String,
}

#[derive(Debug, MultipartForm)]
struct ImageUploadingForm {
    #[multipart(limit = "500KB")]
    image: TempFile,
}

#[derive(Serialize)]
struct ImageUploadingResponse {
    uploaded_file_name: String,
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

    let (mut posts, _total_count, num_pages) = Query::find_posts_of_user_and_status_in_page(conn, user_id, params.status, page, posts_per_page)
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
            post_saving_request.category,
            post_saving_request.the_abstract,
            post_saving_request.text,
            post_saving_request.references).await {
        Ok(saved_post) => saved_post,
        Err(e) => {
            println!("Database error: {:?}", e);

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

    let id = id.into_inner();
    let post_saving_request = post_saving_request_json.into_inner();

    match shencha_text(format!("{} {} {} {} {}", post_saving_request.title, post_saving_request.sharing_path, post_saving_request.tags, post_saving_request.the_abstract, post_saving_request.references).as_str()) {
        Ok(true) => {}
        Ok(false) => {
            return Ok(HttpResponse::Forbidden().finish())
        }
        Err(e) => {
            println!("Shencha error: {:?}", e);

            return Ok(HttpResponse::InternalServerError().finish())
        }
    }

    match shencha_text(post_saving_request.text.as_str()) {
        Ok(true) => {}
        Ok(false) => {
            return Ok(HttpResponse::Forbidden().finish())
        }
        Err(e) => {
            println!("Shencha error: {:?}", e);

            return Ok(HttpResponse::InternalServerError().finish())
        }
    }

    let conn = &data.conn;
    
    let saved_post = match Mutation::update_post_by_id(conn,
            user_id,
            id,
            post_saving_request.title,
            post_saving_request.sharing_path,
            post_saving_request.tags,
            post_saving_request.category,
            post_saving_request.the_abstract,
            post_saving_request.text,
            post_saving_request.references).await {
        Ok(saved_post) => saved_post,
        Err(e) => {
            println!("Database error: {:?}", e);

            return Ok(HttpResponse::InternalServerError().finish())
        }
    };

    // if saved_post.status == STATUS_PUBLISHED {
    //     let page = 1;
    //     let posts_per_page = DEFAULT_POSTS_PER_PAGE;
    
    //     let (posts, total_count, num_pages) = Query::find_posts_of_user_and_status_in_page(conn, user_id, STATUS_PUBLISHED, page, posts_per_page)
    //         .await
    //         .expect("Cannot find posts in page");
    
    //     publish_post_page(&data, user_id, user_nick, user_registering_time, &mut saved_post).await;
    
    //     publish_home_page(&data, user_id, user_nick, user_registering_time, posts, total_count, num_pages, page, posts_per_page).await;
    // }

    Ok(HttpResponse::Ok().json(saved_post.try_into_model().unwrap()))
}

#[put("/posts/{id}/images")]
pub async fn upload_image(
    req: HttpRequest,
    id: web::Path<i32>,
    MultipartForm(form): MultipartForm<ImageUploadingForm>,
) -> Result<HttpResponse, Error> {
    if form.image.size < 1 || form.image.size > 1024 * 500 {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let user_id = match req.headers().get("id") {
        Some(id) => id.to_str().unwrap().parse::<i32>().unwrap(),
        None => {
            return Ok(HttpResponse::NotFound().finish())
        }
    };

    let folder = format!("./web/sololo.cn/usr/share/nginx/html/cndev/_post_images/{}/{}", user_id, id);

    // Just treat all images as png.
    let uploaded_file_name = format!("{}.png", Utc::now().timestamp());

    let _ = std::fs::create_dir_all(&folder);
    println!("{}/{}", folder, uploaded_file_name);
    let mut file = std::fs::File::create(format!("{}/{}", folder, uploaded_file_name)).unwrap();
    let mut content = vec![];
    let mut image_file = form.image.file;
    image_file.read_to_end(&mut content).unwrap();
    file.write_all(&content).unwrap();

    Ok(HttpResponse::Ok().json(ImageUploadingResponse {
        uploaded_file_name: format!("{}/{}/{}", user_id, id, uploaded_file_name),
    }))
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
            println!("Database error: {:?}", e);

            return Ok(HttpResponse::InternalServerError().finish())
        }
    };

    let page = 1;
    let posts_per_page = DEFAULT_POSTS_PER_PAGE;

    let (posts, total_count, num_pages) = Query::find_posts_of_user_and_status_in_page(conn, user_id, STATUS_PUBLISHED, page, posts_per_page)
        .await
        .expect("Cannot find posts in page");

    publish_post_page(&data, user_id, user_nick, user_registering_time, &mut saved_post).await;

    publish_home_page(&data, user_id, user_nick, user_registering_time, posts, total_count, num_pages, page, posts_per_page).await;

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
            println!("Database error: {:?}", e);

            return Ok(HttpResponse::InternalServerError().finish())
        }
    };

    let page = 1;
    let posts_per_page = DEFAULT_POSTS_PER_PAGE;

    let (posts, total_count, num_pages) = Query::find_posts_of_user_and_status_in_page(conn, user_id, STATUS_PUBLISHED, page, posts_per_page)
        .await
        .expect("Cannot find posts in page");

    unpublish_post_page(user_id, &mut saved_post).await;

    publish_home_page(&data, user_id, user_nick, user_registering_time, posts, total_count, num_pages, page, posts_per_page).await;

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
            println!("Database error: {:?}", e);

            return Ok(HttpResponse::InternalServerError().finish())
        }
    };

    let page = 1;
    let posts_per_page = DEFAULT_POSTS_PER_PAGE;

    let (posts, total_count, num_pages) = Query::find_posts_of_user_and_status_in_page(conn, user_id, STATUS_PUBLISHED, page, posts_per_page)
        .await
        .expect("Cannot find posts in page");

    unpublish_post_page(user_id, &mut saved_post).await;
    // publish_post_page(&data, user_id, user_nick, user_registering_time, &mut saved_post).await;

    publish_home_page(&data, user_id, user_nick, user_registering_time, posts, total_count, num_pages, page, posts_per_page).await;

    Ok(HttpResponse::Ok().finish())
}

async fn publish_post_page(data: &web::Data<AppState>, author_id: i32, author_nick: &str, author_registering_time: i64,
        post: &mut post::Model) -> bool {
    println!("Publishing post page {} for user {}", post.id, author_id);

    post.updated_at_formatted = post.updated_at.format("%Y-%m-%d %H:%M:%S").to_string();

    if post.sharing_path.len() > 0 {
        post.id_or_sharing_path = format!("{}", post.sharing_path);
    } else {
        post.id_or_sharing_path = format!("{}", post.id);
    }

    let mut post_to_render = post.clone();
    post_to_render.title = escape_html(&post_to_render.title);
    post_to_render.the_abstract = escape_html(post_to_render.the_abstract.as_str()).replace("\n", "").replace("\r", "");
    post_to_render.text = escape_html(post_to_render.text.as_str());
    post_to_render.references = escape_html(post_to_render.references.as_str());

    let template = &data.templates;
    let mut ctx = tera::Context::new();
    ctx.insert("template", "post");
    ctx.insert("author_id", &author_id);
    ctx.insert("author_nick", author_nick);
    ctx.insert("author_registering_time", &format!("{}", author_registering_time));
    ctx.insert("post", &post_to_render);

    let body = template
        .render("post.html.tera", &ctx)
        .unwrap()
        .into_bytes();

    let folder_by_id_path = format!("./web/cn.dev/usr/share/nginx/html/index-and-homes/root/{}", author_id);
    let _ = std::fs::create_dir_all(&folder_by_id_path);

    let folder_by_nick_path = format!("./web/cn.dev/usr/share/nginx/html/index-and-homes/root/{}", author_nick);
    let _ = std::os::unix::fs::symlink(format!("./{}", author_id), folder_by_nick_path);

    let file_by_id_path = format!("./web/cn.dev/usr/share/nginx/html/index-and-homes/root/{}/{}.html", author_id, post.id);
    let mut file = std::fs::File::create(&file_by_id_path).unwrap();
    file.write_all(&body).unwrap();

    if post.old_sharing_path.len() > 0 && post.old_sharing_path != post.sharing_path {
        let _ = std::fs::remove_file(format!("./web/cn.dev/usr/share/nginx/html/index-and-homes/root/{}/{}.html", author_id, post.old_sharing_path));
    }

    let file_by_sharing_path = format!("./web/cn.dev/usr/share/nginx/html/index-and-homes/root/{}/{}.html", author_id, post.sharing_path);
    let _ = std::os::unix::fs::symlink(format!("./{}.html", post.id), file_by_sharing_path);

    println!("Published post page {} for user {}", post.id, author_id);

    false
}

async fn unpublish_post_page(author_id: i32, post: &mut post::Model) -> bool {
    println!("Unpublishing post page {} for user {}", post.id, author_id);

    let _ = std::fs::remove_file(format!("./web/cn.dev/usr/share/nginx/html/index-and-homes/root/{}/{}.html", author_id, post.id));

    println!("Unpublished post page {} for user {}", post.id, author_id);

    false
}

pub async fn publish_home_page(data: &web::Data<AppState>, author_id: i32, author_nick: &str, author_registering_time: i64,
        mut posts: Vec<post::Model>, total_count: u64, num_pages: u64, page: u64, posts_per_page: u64) -> bool {
    println!("Publishing home page for user {}", author_id);

    for post in posts.iter_mut() {
        post.title = escape_html(&post.title);

        post.updated_at_formatted = post.updated_at.format("%Y-%m-%d %H:%M:%S").to_string();

        if post.sharing_path.len() > 0 {
            post.id_or_sharing_path = format!("{}", post.sharing_path);
        } else {
            post.id_or_sharing_path = format!("{}", post.id);
        }
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

    let file_by_id_path = format!("./web/cn.dev/usr/share/nginx/html/index-and-homes/root/{}.html", author_id);
    let mut file = std::fs::File::create(&file_by_id_path).unwrap();
    file.write_all(&body).unwrap();

    if author_nick.len() > 0 {
        let file_by_nick_path = format!("./web/cn.dev/usr/share/nginx/html/index-and-homes/root/{}.html", author_nick);
        let _ = std::os::unix::fs::symlink(format!("./{}.html", author_id), file_by_nick_path);
    }

    println!("Published home page of user {}", author_id);

    false
}

fn escape_html(html: &str) -> String {
    html.replace("<", "&lt;").replace(">", "&gt;")
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