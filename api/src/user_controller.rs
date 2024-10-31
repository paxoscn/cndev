
use std::{io::Read, ops::Index};

use cndev_service::{
    Mutation, Query,
};

use entity::user;
use rand::Rng;
use redis::Commands;
use serde::{Serialize, Deserialize};
use crate::controllers::AppState;

use crate::shencha;

use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};

use std::env;

use std::io::Write;

use actix_web::{
    error, get, post, put, delete, web, Error, HttpRequest, HttpResponse, Result,
};

use actix_multipart::form::{tempfile::TempFile, MultipartForm};

use std::net::IpAddr;

use chrono::{format, DateTime, Utc};

use crate::post_controller::publish_home_page;

use entity::post::STATUS_PUBLISHED;

const DEFAULT_POSTS_PER_PAGE: u64 = 100;

#[derive(Deserialize)]
struct SmsSendingRequest {
    tel: String,
}

#[derive(Deserialize)]
struct TokenGrantingRequest {
    tel: String,
    sms_code: String,
}

#[derive(Serialize)]
struct TokenGrantingResponse {
    token: String,
    id: i32,
    nick: String,
    registering_time: i64,
    user_created: bool,
}

#[derive(Debug, Deserialize)]
pub struct Params {
    page: Option<u64>,
    users_per_page: Option<u64>,
}

#[derive(Deserialize)]
struct NickChangingRequest {
    nick: String,
}

#[derive(Debug, Serialize)]
struct Claims {
    id: i32,
    nick: String,
    registering_time: i64,
    exp: usize,
}

#[derive(Debug, MultipartForm)]
struct AvatarUploadingForm {
    #[multipart(limit = "100KB")]
    avatar: TempFile,
}

#[derive(Serialize)]
struct AvatarUploadingResponse {
    uploaded_file_name: String,
}

#[get("/users/")]
async fn list(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let template = &data.templates;
    let conn = &data.conn;

    // get params
    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let page = params.page.unwrap_or(1);
    let users_per_page = params.users_per_page.unwrap_or(DEFAULT_POSTS_PER_PAGE);

    let (users, num_pages) = Query::find_users_in_page(conn, page, users_per_page)
        .await
        .expect("Cannot find users in page");

    // Return users as JSON
    Ok(HttpResponse::Ok().json(users))
    // let mut ctx = tera::Context::new();
    // ctx.insert("users", &users);
    // ctx.insert("page", &page);
    // ctx.insert("users_per_page", &users_per_page);
    // ctx.insert("num_pages", &num_pages);

    // let body = template
    //     .render("index.html.tera", &ctx)
    //     .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    // Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[get("/users/new")]
async fn new(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let template = &data.templates;
    let ctx = tera::Context::new();
    let body = template
        .render("new.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

// curl -v -XPOST -H 'Content-Type: application/json' 'http://127.0.0.1:8000/users/commands/sms-sending' -d '{ "tel": "13699124376" }'
#[post("/users/commands/sms-sending")]
async fn send_sms(
    req: HttpRequest,
    data: web::Data<AppState>,
    sms_sending_request_json: web::Json<SmsSendingRequest>,
) -> Result<HttpResponse, Error> {
    let connection_info = req.connection_info();

    let client_ip = connection_info.realip_remote_addr().unwrap();
    if is_internal_ip(client_ip) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let sms_sending_request = sms_sending_request_json.into_inner();

    match check_sms_sending_times(&data.redis, client_ip) {
        Some(res) => {
            return res;
        }
        None => {}
    }

    // TODO Init on startup.
    let aliyun_sms_region = env::var("ALIYUN_SMS_REGION").expect("ALIYUN_SMS_REGION is not set in .env file");
    let aliyun_sms_ak = env::var("ALIYUN_SMS_AK").expect("ALIYUN_SMS_AK is not set in .env file");
    let aliyun_sms_sk = env::var("ALIYUN_SMS_SK").expect("ALIYUN_SMS_SK is not set");

    let mut aliyun_sms_client = alibaba_cloud_sdk_rust::services::dysmsapi::Client::NewClientWithAccessKey(
        aliyun_sms_region.as_str(),
        aliyun_sms_ak.as_str(),
        aliyun_sms_sk.as_str(),
    )?;

    match cache_and_send_sms(&data.redis, aliyun_sms_client, &sms_sending_request.tel) {
        Some(res) => {
            return res;
        }
        None => {}
    }

    println!("sending to {}", sms_sending_request.tel);

    Ok(HttpResponse::Ok().finish())
}

// curl -v -XPOST -H 'Content-Type: application/json' 'http://127.0.0.1:8000/tokens' -d '{ "tel": "13666666666", "sms_code": "123456" }'
// curl -XPOST -H 'Content-Type: application/json' 'http://127.0.0.1:8000/tokens' -d '{ "tel": "13666666666", "sms_code": "123456" }' | sed 's/.*"token":"\([^"]*\)".*/\1/g' > /tmp/token
#[post("/tokens")]
async fn grant_token(
    data: web::Data<AppState>,
    token_granting_request_json: web::Json<TokenGrantingRequest>,
) -> Result<HttpResponse, Error> {
    let token_granting_request = token_granting_request_json.into_inner();

    match check_sms_code(&data.redis, &token_granting_request.tel, &token_granting_request.sms_code) {
        Some(res) => {
            println!("Failed to check SMS code");

            return res;
        }
        None => {}
    }

    let conn = &data.conn;

    let mut user_created = false;

    let user: Option<user::Model> = match Query::find_user_by_tel(conn, &token_granting_request.tel).await {
        Ok(existing_user_) => {
            match existing_user_ {
                Some(existing_user) => {
                    Option::Some(existing_user)
                }
                None => {
                    match Mutation::create_user(conn, &token_granting_request.tel).await {
                        Ok(_) => {
                            user_created = true;
                        }
                        Err(e) => {
                            println!("Database error: {:?}", e);
                        }
                    }
                    match Query::find_user_by_tel(conn, &token_granting_request.tel).await {
                        Ok(found_user) => {
                            found_user
                        }
                        Err(e) => {
                            println!("Database error: {:?}", e);
                            
                            Option::None
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Database error: {:?}", e);

            Option::None
        }
    };

    match user {
        Some(mut user) => {
            if user_created {
                user.nick = Some(format!("user{}", user.id));
    
                Mutation::update_user_by_id(conn, user.id, &user)
                .await
                .expect("could not edit user");
            }

            let nick = user.nick.unwrap_or(String::new());
            
            let my_claims = Claims {
                id: user.id,
                nick: nick.to_owned(),
                registering_time: user.created_at.timestamp(),
                exp: (Utc::now().timestamp() + 86400 * 365) as usize, // UNIX timestamp for expiration
            };
            
            let jwt_secret = env::var("APP_VERSION").expect("APP_VERSION is not set");
            
            let token = encode(&Header::new(Algorithm::HS256), &my_claims, &EncodingKey::from_secret(jwt_secret.as_ref())).unwrap();
            
            println!("{}", token);
        
            Ok(HttpResponse::Created().json(TokenGrantingResponse {
                token: token,
                id: user.id,
                nick: nick,
                registering_time: user.created_at.timestamp(),
                user_created: user_created,
            }))
        }
        None => Ok(HttpResponse::InternalServerError().finish())
    }
}

#[get("/users/{nick}")]
async fn load(data: web::Data<AppState>, nick: web::Path<String>) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let nick = nick.into_inner();

    let user = Query::find_user_by_nick(conn, nick.clone())
    .await
    .expect("could not find user")
    .unwrap_or_else(|| panic!("could not find user with nick {nick}"));

    Ok(HttpResponse::Ok().json(user))
}

#[post("/users/{id}")]
async fn update(
    data: web::Data<AppState>,
    id: web::Path<i32>,
    user_form: web::Form<user::Model>,
) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let form = user_form.into_inner();
    let id = id.into_inner();

    Mutation::update_user_by_id(conn, id, &form)
        .await
        .expect("could not edit user");

    Ok(HttpResponse::Found()
        .append_header(("location", "/"))
        .finish())
}

#[post("/users/delete/{id}")]
async fn delete(data: web::Data<AppState>, id: web::Path<i32>) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let id = id.into_inner();

    Mutation::delete_user(conn, id)
        .await
        .expect("could not delete user");

    Ok(HttpResponse::Found()
        .append_header(("location", "/"))
        .finish())
}

#[put("/settings/avatar")]
pub async fn upload_avatar(
    req: HttpRequest,
    data: web::Data<AppState>,
    MultipartForm(form): MultipartForm<AvatarUploadingForm>,
) -> Result<HttpResponse, Error> {
    if form.avatar.size < 1 || form.avatar.size > 1024 * 100 {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let user_id = match req.headers().get("id") {
        Some(id) => id.to_str().unwrap().parse::<i32>().unwrap(),
        None => {
            return Ok(HttpResponse::NotFound().finish())
        }
    };

    // Just treat all images as png.
    let uploaded_file_name = format!("{}.png", user_id);
    // let uploaded_file_name = match form.avatar.file_name {
    //     Some(file_name) => {
    //         if file_name.contains(".") {
    //             let file_name_split = file_name.split(".").collect::<Vec<&str>>();
    //             let file_extension = file_name_split.last().unwrap_or(&"png").to_lowercase();

    //             if file_extension != "png" && file_extension != "jpg" && file_extension != "jpeg" && file_extension != "gif" && file_extension != "webp" {
    //                 return Ok(HttpResponse::BadRequest().finish());
    //             }
                
    //             format!("{}.{}", user_id, file_extension)
    //         } else {
    //             format!("{}.png", user_id)
    //         }
    //     }
    //     None => {
    //         format!("{}.png", user_id)
    //     }
    // };

    let mut file = std::fs::File::create(format!("./web/sololo.cn/usr/share/nginx/html/cndev/_avatars/{}", uploaded_file_name)).unwrap();
    let mut content = vec![];
    let mut avatar_file = form.avatar.file;
    avatar_file.read_to_end(&mut content).unwrap();
    file.write_all(&content).unwrap();

    Ok(HttpResponse::Ok().json(AvatarUploadingResponse {
        uploaded_file_name: uploaded_file_name,
    }))
}

#[delete("/settings/avatar")]
pub async fn remove_avatar(
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user_id = match req.headers().get("id") {
        Some(id) => id.to_str().unwrap().parse::<i32>().unwrap(),
        None => {
            return Ok(HttpResponse::NotFound().finish())
        }
    };

    // Just treat all images as png.
    let uploaded_file_name = format!("{}.png", user_id);
    // let uploaded_file_name = match form.avatar.file_name {
    //     Some(file_name) => {
    //         if file_name.contains(".") {
    //             let file_name_split = file_name.split(".").collect::<Vec<&str>>();
    //             let file_extension = file_name_split.last().unwrap_or(&"png").to_lowercase();

    //             if file_extension != "png" && file_extension != "jpg" && file_extension != "jpeg" && file_extension != "gif" && file_extension != "webp" {
    //                 return Ok(HttpResponse::BadRequest().finish());
    //             }
                
    //             format!("{}.{}", user_id, file_extension)
    //         } else {
    //             format!("{}.png", user_id)
    //         }
    //     }
    //     None => {
    //         format!("{}.png", user_id)
    //     }
    // };

    let file_path = format!("./web/sololo.cn/usr/share/nginx/html/cndev/_avatars/{}", uploaded_file_name);
    if std::fs::remove_file(&file_path).is_err() {
        return Ok(HttpResponse::InternalServerError().finish());
    }

    Ok(HttpResponse::Ok().finish())
}

#[put("/settings")]
async fn change_nick(
    req: HttpRequest,
    data: web::Data<AppState>,
    nick_changing_request_json: web::Json<NickChangingRequest>,
) -> Result<HttpResponse, Error> {
    let user_id = match req.headers().get("id") {
        Some(id) => id.to_str().unwrap().parse::<i32>().unwrap(),
        None => {
            return Ok(HttpResponse::NotFound().finish())
        }
    };
    let user_old_nick = req.headers().get("nick").unwrap().to_str().unwrap();
    let user_registering_time = req.headers().get("reg").unwrap().to_str().unwrap().parse::<i64>().unwrap();

    let nick_changing_request = nick_changing_request_json.into_inner();
    let user_new_nick = nick_changing_request.nick.trim().to_lowercase()
            .replace(|c: char| c != '-' && !c.is_alphanumeric() && !c.is_numeric(), "");

    // Not allowed to be empty.
    if user_new_nick.len() < 1 || user_new_nick == user_old_nick {
        return Ok(HttpResponse::Ok().finish())
    }

    match shencha_nick(user_new_nick.as_str()) {
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
    
    match Mutation::change_nick(conn,
            user_id,
            user_new_nick.to_owned()).await {
        Ok(_) => {},
        Err(e) => {
            println!("Database error: {:?}", e);

            return Ok(HttpResponse::InternalServerError().finish())
        }
    };

    if user_old_nick.len() > 0 {
        let _ = std::fs::remove_file(format!("./web/cn.dev/usr/share/nginx/html/index-and-homes/root/{}", user_old_nick));
        let _ = std::fs::remove_file(format!("./web/cn.dev/usr/share/nginx/html/index-and-homes/root/{}.html", user_old_nick));

        let page = 1;
        let posts_per_page = DEFAULT_POSTS_PER_PAGE;

        let (posts, total_count, num_pages) = Query::find_posts_of_user_and_status_in_page(conn, user_id, STATUS_PUBLISHED, page, posts_per_page)
            .await
            .expect("Cannot find posts in page");

        publish_home_page(&data, user_id, user_new_nick.as_str(), user_registering_time, posts, total_count, num_pages, page, posts_per_page).await;

        let folder_by_nick_path = format!("./web/cn.dev/usr/share/nginx/html/index-and-homes/root/{}", user_new_nick);
        let _ = std::os::unix::fs::symlink(format!("./{}", user_id), folder_by_nick_path);
    }

    let my_claims = Claims {
        id: user_id,
        nick: user_new_nick.to_owned(),
        registering_time: user_registering_time,
        exp: (Utc::now().timestamp() + 86400 * 365) as usize, // UNIX timestamp for expiration
    };
    
    let jwt_secret = env::var("APP_VERSION").expect("APP_VERSION is not set");
    
    let token = encode(&Header::new(Algorithm::HS256), &my_claims, &EncodingKey::from_secret(jwt_secret.as_ref())).unwrap();
    
    println!("{}", token);

    Ok(HttpResponse::Ok().json(TokenGrantingResponse {
        token: token,
        id: user_id,
        nick: user_new_nick.to_owned(),
        registering_time: user_registering_time,
        user_created: false,
    }))
}

fn is_internal_ip(ip: &str) -> bool {
    if let Ok(ip) = ip.parse::<IpAddr>() {
        return match ip {
            IpAddr::V4(ip4) => {
                let octets = ip4.octets();
                (octets[0] == 10)
                    || (octets[0] == 172 && (16..=31).contains(&octets[1]))
                    || (octets[0] == 192 && octets[1] == 168)
            }
            IpAddr::V6(_) => false, // Assuming IPv6 addresses are not internal
        };
    }
    false
}

fn check_sms_sending_times(redis: &redis::Client, client_ip: &str) -> Option<Result<HttpResponse, Error>> {
    match redis.get_connection() {
        Ok(mut conn) => {
            let sms_sending_times: i32 = conn.get(format!("sms_sending_times-{}", client_ip)).unwrap_or(0);

            // if sms_sending_times > 5 {
            //     return Some(Ok(HttpResponse::Forbidden().finish()));
            // }

            conn.incr(format!("sms_sending_times-{}", client_ip), 1).unwrap_or(());

            conn.expire(format!("sms_sending_times-{}", client_ip), 3600).unwrap_or(());

            return None;
        }
        Err(_) => {
            return Some(Ok(HttpResponse::InternalServerError().finish()));
        }
    }
}

fn cache_and_send_sms(redis: &redis::Client, mut aliyun_sms_client: alibaba_cloud_sdk_rust::services::dysmsapi::Client, tel: &str) -> Option<Result<HttpResponse, Error>> {
    let sms_code = generate_sms_code();

    println!("sending {} to {}", sms_code, tel);

    match redis.get_connection() {
        Ok(mut conn) => {
            conn.set(format!("sms_code-{}", tel), sms_code.to_owned()).unwrap_or(0);

            conn.expire(format!("sms_code-{}", tel), 180).unwrap_or(());

            // TODO Async.
            let aliyun_sms_template = env::var("ALIYUN_SMS_TEMPLATE").expect("ALIYUN_SMS_TEMPLATE is not set in .env file");
            let aliyun_sms_signature = env::var("ALIYUN_SMS_SIGNATURE").expect("ALIYUN_SMS_SIGNATURE is not set in .env file");

            let mut request = alibaba_cloud_sdk_rust::services::dysmsapi::CreateSendSmsRequest();
            request.PhoneNumbers = String::from(tel);
            request.SignName = aliyun_sms_signature;
            request.TemplateCode = aliyun_sms_template;
            request.TemplateParam = format!("{{\"code\":\"{}\"}}", sms_code);
            let response = aliyun_sms_client.SendSms(&mut request).ok()?;
            println!("{:?}", &response);

            return None;
        }
        Err(_) => {
            return Some(Ok(HttpResponse::InternalServerError().finish()));
        }
    }
}

fn check_sms_code(redis: &redis::Client, tel: &str, sms_code: &str) -> Option<Result<HttpResponse, Error>> {
    match redis.get_connection() {
        Ok(mut conn) => {
            let valid_sms_code: String = conn.get(format!("sms_code-{}", tel)).unwrap_or("".to_owned());

            if valid_sms_code.len() < 1 || valid_sms_code != sms_code {
                return Some(Ok(HttpResponse::Forbidden().finish()));
            }

            conn.del(format!("sms_code-{}", tel)).unwrap_or(0);

            return None;
        }
        Err(_) => {
            return Some(Ok(HttpResponse::InternalServerError().finish()));
        }
    }
}

fn generate_sms_code() -> String {
    let mut rng = rand::thread_rng();
    let base: i32 = 9;
    let min: i32 = base.pow(6);
    let max: i32 = min * 2 - 1;
    let code: i32 = rng.gen_range(min..max);
    return to_base9(code)[1..].to_string().replace("4", "9");
}

fn to_base9(mut num: i32) -> String {
    let mut result = String::new();
    while num > 0 {
        let digit = num % 9;
        result.push_str(&digit.to_string());
        num /= 9;
    }

    result.chars().rev().collect::<String>()
}

fn shencha_nick(nick: &str) -> Result<bool, std::io::Error> {
    // TODO Init on startup.
    let aliyun_shencha_region = env::var("ALIYUN_SHENCHA_REGION").expect("ALIYUN_SHENCHA_REGION is not set in .env file");
    let aliyun_shencha_ak = env::var("ALIYUN_SHENCHA_AK").expect("ALIYUN_SHENCHA_AK is not set in .env file");
    let aliyun_shencha_sk = env::var("ALIYUN_SHENCHA_SK").expect("ALIYUN_SHENCHA_SK is not set");

    let mut aliyun_client = alibaba_cloud_sdk_rust::services::dysmsapi::Client::NewClientWithAccessKey(
        aliyun_shencha_region.as_str(),
        aliyun_shencha_ak.as_str(),
        aliyun_shencha_sk.as_str(),
    )?;

    shencha::shencha(&mut aliyun_client, "nickname_detection", nick)
}