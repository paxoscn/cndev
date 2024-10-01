mod controllers;
mod post_controller;
mod user_controller;
pub mod shencha;

use cndev_service::sea_orm::Database;
use actix_files::Files as Fs;
use actix_web::{
    error, middleware, web, dev::Service, dev::ServiceResponse, App, Error, HttpRequest, HttpResponse, HttpServer, Result, http::header::HeaderName, http::header::HeaderValue,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use futures_util::future::FutureExt;

use crate::controllers::AppState;

use listenfd::ListenFd;
use migration::{Migrator, MigratorTrait};
use serde::{Deserialize, Serialize};
use std::env;
use tera::Tera;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    id: i32,
    nick: String,
    registering_time: i64,
    exp: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct FlashData {
    kind: String,
    message: String,
}

async fn not_found(data: web::Data<AppState>, request: HttpRequest) -> Result<HttpResponse, Error> {
    let mut ctx = tera::Context::new();
    ctx.insert("uri", request.uri().path());

    let template = &data.templates;
    let body = template
        .render("error/404.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[actix_web::main]
async fn start() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    // get env vars
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL is not set in .env file");
    let server_url = format!("{host}:{port}");

    // establish connection to database and apply migrations
    // -> create post table if not exists
    let conn = Database::connect(&db_url).await.unwrap();
    Migrator::up(&conn, None).await.unwrap();

    // Init a null redis client
    let redis: redis::Client = redis::Client::open(redis_url).unwrap();

    // load tera templates and build app state
    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
    let state = AppState { templates, conn, redis};

    // create server and try to serve over socket if possible
    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        App::new()
            .service(Fs::new("/static", "./api/static"))
            .app_data(web::Data::new(state.clone()))
            .wrap(middleware::Logger::default()) // enable logger
            .wrap_fn(|mut req, srv| {
                let authorization = req.headers().get("Authorization").and_then(|value| value.to_str().ok());
                // let xxx = token.unwrap();
                // println!("token {xxx}");

                match authorization {
                    Some(authorization) => {
                        // Trimming the "Bearer " prefix.
                        let token = &authorization[7..];
                        let mut validation = Validation::new(Algorithm::HS256);
                        // validation.sub = Some("b@b.com".to_string());
                        // validation.set_audience(&["me"]);
                        validation.set_required_spec_claims(&["id", "nick", "exp"]);

                        let jwt_secret = env::var("APP_VERSION").expect("APP_VERSION is not set");

                        match decode::<Claims>(&token, &DecodingKey::from_secret(jwt_secret.as_bytes()), &validation) {
                            Ok(token_data) => {
                                // TODO Checking if the token is expired

                                req.headers_mut().insert(HeaderName::from_bytes("id".as_bytes()).unwrap(),
                                        HeaderValue::from_bytes(token_data.claims.id.to_string().as_bytes()).unwrap());
                                req.headers_mut().insert(HeaderName::from_bytes("nick".as_bytes()).unwrap(),
                                        HeaderValue::from_bytes(token_data.claims.nick.as_bytes()).unwrap());
                                req.headers_mut().insert(HeaderName::from_bytes("reg".as_bytes()).unwrap(),
                                        HeaderValue::from_bytes(format!("{}", token_data.claims.registering_time).as_bytes()).unwrap());

                                let fut = srv.call(req);
                        
                                Box::pin(async move {
                                    let res = fut.await?;
                                    
                                    // Map to L type
                                    Ok(res.map_into_left_body())
                                }).boxed_local()
                            },
                            Err(e) => {
                                let xxx = e.to_string();
                                println!("Error: {xxx}");
                                let http_res = HttpResponse::Unauthorized().finish();
                                let (http_req, _) = req.into_parts();
                                let res = ServiceResponse::new(http_req, http_res);
                                return (async move { Ok(res.map_into_right_body()) }).boxed_local();
                            },
                        }
                    },
                    None => {
                        let path = req.path();

                        if path == "/tokens" || path == "/users/commands/sms-sending" {
                            let fut = srv.call(req);
                    
                            Box::pin(async move {
                                let res = fut.await?;
                                
                                // Map to L type
                                Ok(res.map_into_left_body())
                            }).boxed_local()
                        } else {
                            let http_res = HttpResponse::Unauthorized().finish();
                            let (http_req, _) = req.into_parts();
                            let res = ServiceResponse::new(http_req, http_res);
                            return (async move { Ok(res.map_into_right_body()) }).boxed_local();
                        }
                    },
                }
            })
            .default_service(web::route().to(not_found))
            .configure(init)
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => server.bind(&server_url)?,
    };

    println!("Starting server at {server_url}");
    server.run().await?;

    Ok(())
}

fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(post_controller::list);
    cfg.service(post_controller::load);
    // cfg.service(post_controller::new);
    cfg.service(post_controller::create);
    // cfg.service(post_controller::edit);
    cfg.service(post_controller::update);
    cfg.service(post_controller::delete);
    cfg.service(post_controller::publish);
    cfg.service(post_controller::unpublish);

    cfg.service(user_controller::grant_token);
    cfg.service(user_controller::list);
    cfg.service(user_controller::new);
    cfg.service(user_controller::edit);
    cfg.service(user_controller::update);
    cfg.service(user_controller::delete);
    cfg.service(user_controller::send_sms);
    cfg.service(user_controller::change_nick);
    cfg.service(user_controller::upload_avatar);
    cfg.service(user_controller::remove_avatar);
}

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
