
use std::ops::Index;

use cndev_service::{
    Mutation, Query,
};

use entity::user;
use redis::Commands;
use serde::{Serialize, Deserialize};
use crate::controllers::AppState;

use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};

use actix_web::{
    error, get, post, put, web, Error, HttpRequest, HttpResponse, Result,
};

use std::net::IpAddr;

use chrono::{DateTime, Utc};

const DEFAULT_POSTS_PER_PAGE: u64 = 5;

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

#[derive(Debug, Serialize)]
struct Claims {
    id: i32,
    nick: String,
    registering_time: i64,
    exp: usize,
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

    match cache_and_send_sms(&data.redis, &sms_sending_request.tel) {
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

    // match check_sms_code(&data.redis, &token_granting_request.tel, &token_granting_request.sms_code) {
    //     Some(res) => {
    //         return res;
    //     }
    //     None => {}
    // }

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
                            print!("Database error: {:?}", e);
                        }
                    }
                    match Query::find_user_by_tel(conn, &token_granting_request.tel).await {
                        Ok(found_user) => {
                            found_user
                        }
                        Err(e) => {
                            print!("Database error: {:?}", e);
                            
                            Option::None
                        }
                    }
                }
            }
        }
        Err(e) => {
            print!("Database error: {:?}", e);

            Option::None
        }
    };

    match user {
        Some(user) => {
            let my_claims = Claims {
                id: user.id,
                nick: user.nick.to_owned(),
                registering_time: user.created_at.timestamp(),
                exp: (Utc::now().timestamp() + 86400 * 365) as usize, // UNIX timestamp for expiration
            };
            
            let key = "secret";
            
            let token = encode(&Header::new(Algorithm::HS256), &my_claims, &EncodingKey::from_secret(key.as_ref())).unwrap();
            
            println!("{}", token);
        
            Ok(HttpResponse::Created().json(TokenGrantingResponse {
                token: token,
                id: user.id,
                nick: user.nick,
                registering_time: user.created_at.timestamp(),
                user_created: user_created,
            }))
        }
        None => Ok(HttpResponse::InternalServerError().finish())
    }
}

#[get("/users/{id}")]
async fn edit(data: web::Data<AppState>, id: web::Path<i32>) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let template = &data.templates;
    let id = id.into_inner();

    let user: user::Model = Query::find_user_by_id(conn, id)
        .await
        .expect("could not find user")
        .unwrap_or_else(|| panic!("could not find user with id {id}"));

    let mut ctx = tera::Context::new();
    ctx.insert("user", &user);

    let body = template
        .render("edit.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
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

    Mutation::update_user_by_id(conn, id, form)
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

fn cache_and_send_sms(redis: &redis::Client, tel: &str) -> Option<Result<HttpResponse, Error>> {
    let sms_code = "123456";

    match redis.get_connection() {
        Ok(mut conn) => {
            conn.set(format!("sms_code-{}", tel), sms_code).unwrap_or(0);

            conn.expire(format!("sms_code-{}", tel), 180).unwrap_or(());

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
