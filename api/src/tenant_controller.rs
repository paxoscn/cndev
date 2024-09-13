
use std::ops::Index;

use cndev_service::{
    Mutation, Query,
};

use entity::tenant;
use redis::Commands;
use serde::{Serialize, Deserialize};
use crate::controllers::AppState;

use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};

use actix_web::{
    error, get, post, put, web, Error, HttpRequest, HttpResponse, Result,
};

use std::net::IpAddr;

const DEFAULT_POSTS_PER_PAGE: u64 = 5;

#[derive(Deserialize)]
struct SmsSendingRequest {
    tel: String,
}

#[derive(Deserialize)]
struct RegistrationRequest {
    tel: String,
    sms_code: String,
    tenant: tenant::Model,
}

#[derive(Debug, Deserialize)]
pub struct Params {
    page: Option<u64>,
    tenants_per_page: Option<u64>,
}

#[derive(Debug, Serialize)]
struct Claims {
    sub: String,
    company: String,
    exp: usize,
}

// curl -v -XPOST -H 'Content-Type: application/json' 'http://127.0.0.1:8000/auth' -d '{ "name": "xxx", "status": 1}'
#[post("/auth")]
async fn auth(
    data: web::Data<AppState>,
    tenant_json: web::Json<tenant::Model>,
) -> Result<HttpResponse, Error> {
    let my_claims = Claims {
        sub: "b@b.com".to_owned(),
        company: "ACME".to_owned(),
        exp: 1607531332, // UNIX timestamp for expiration
    };
    
    let key = "secretsecretsecretsecret11111111";
    
    let token = encode(&Header::new(Algorithm::HS256), &my_claims, &EncodingKey::from_secret(key.as_ref())).unwrap();
    
    println!("{}", token);

    Ok(HttpResponse::Created().finish())
}

#[get("/tenants/")]
async fn list(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let template = &data.templates;
    let conn = &data.conn;

    // get params
    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let page = params.page.unwrap_or(1);
    let tenants_per_page = params.tenants_per_page.unwrap_or(DEFAULT_POSTS_PER_PAGE);

    let (tenants, num_pages) = Query::find_tenants_in_page(conn, page, tenants_per_page)
        .await
        .expect("Cannot find tenants in page");

    // Return tenants as JSON
    Ok(HttpResponse::Ok().json(tenants))
    // let mut ctx = tera::Context::new();
    // ctx.insert("tenants", &tenants);
    // ctx.insert("page", &page);
    // ctx.insert("tenants_per_page", &tenants_per_page);
    // ctx.insert("num_pages", &num_pages);

    // let body = template
    //     .render("index.html.tera", &ctx)
    //     .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    // Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[get("/tenants/new")]
async fn new(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let template = &data.templates;
    let ctx = tera::Context::new();
    let body = template
        .render("new.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

// curl -v -XPOST -H 'Content-Type: application/json' 'http://127.0.0.1:8000/tenants/commands/sms-sending' -d '{ "tel": "13699124376" }'
#[post("/tenants/commands/sms-sending")]
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

// curl -v -XPOST -H 'Content-Type: application/json' 'http://127.0.0.1:8000/tenants/' -d '{ "name": "xxx", "status": 1 }'
#[post("/tenants/")]
async fn register(
    data: web::Data<AppState>,
    registration_request_json: web::Json<RegistrationRequest>,
) -> Result<HttpResponse, Error> {
    let registration_request = registration_request_json.into_inner();

    match check_sms_code(&data.redis, &registration_request.tel, &registration_request.sms_code) {
        Some(res) => {
            return res;
        }
        None => {}
    }

    let conn = &data.conn;

    Mutation::create_tenant(conn, &registration_request.tenant)
        .await
        .expect("could not insert tenant");

    Ok(HttpResponse::Created()
        .json(registration_request.tenant))
}

#[get("/tenants/{id}")]
async fn edit(data: web::Data<AppState>, id: web::Path<i32>) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let template = &data.templates;
    let id = id.into_inner();

    let tenant: tenant::Model = Query::find_tenant_by_id(conn, id)
        .await
        .expect("could not find tenant")
        .unwrap_or_else(|| panic!("could not find tenant with id {id}"));

    let mut ctx = tera::Context::new();
    ctx.insert("tenant", &tenant);

    let body = template
        .render("edit.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[post("/tenants/{id}")]
async fn update(
    data: web::Data<AppState>,
    id: web::Path<i32>,
    tenant_form: web::Form<tenant::Model>,
) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let form = tenant_form.into_inner();
    let id = id.into_inner();

    Mutation::update_tenant_by_id(conn, id, form)
        .await
        .expect("could not edit tenant");

    Ok(HttpResponse::Found()
        .append_header(("location", "/"))
        .finish())
}

#[post("/tenants/delete/{id}")]
async fn delete(data: web::Data<AppState>, id: web::Path<i32>) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let id = id.into_inner();

    Mutation::delete_tenant(conn, id)
        .await
        .expect("could not delete tenant");

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

            if sms_sending_times > 5 {
                return Some(Ok(HttpResponse::Forbidden().finish()));
            }

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
            let valid_sms_code: &str = conn.get(format!("sms_code-{}", tel)).unwrap_or("");

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
