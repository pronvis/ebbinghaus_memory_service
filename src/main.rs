extern crate actix_web;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate failure;
extern crate reqwest;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use std::env;
use std::time::Duration;

mod data;
mod db_actions;
mod handlers;
mod models;
mod phase;
mod scheduler;
mod schema;

use actix_rt;
use actix_web::{get, middleware, post, App, Error, HttpServer};
use actix_web::{web, HttpResponse};
use data::*;
use log::{debug, error, info};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Debug, Fail)]
pub enum StartError {
    #[fail(display = "no '.env' file")]
    NoEnvFile,
    #[fail(display = "DATABASE_URL must be set")]
    NoDataBaseUrl,
    #[fail(display = "connection to db fail")]
    NoDatbaseConnection,
    #[fail(display = "failed to bind to address '{}'", address)]
    FailedBind { address: String },
    #[fail(display = "runtime error")]
    RuntimeError,
}

#[actix_rt::main]
async fn main() -> Result<(), failure::Error> {
    std::env::set_var(
        "RUST_LOG",
        "ebbinghaus_memory_service=debug,actix_web=error",
    );
    env_logger::init();
    dotenv().map_err(|_| StartError::NoEnvFile)?;

    let database_url = env::var("DATABASE_URL").map_err(|_| StartError::NoDataBaseUrl)?;
    debug!("Trying to establish DB connection to '{}'", database_url);
    let db_connection_manager = ConnectionManager::<PgConnection>::new(database_url);
    let db_pool = r2d2::Pool::builder()
        .build(db_connection_manager)
        .map_err(|_| StartError::NoDatbaseConnection)?;

    let conn = db_pool.get().map_err(|_| StartError::NoDatbaseConnection)?;
    let phases = db_actions::get_phases(&conn)?;

    scheduler::start_checking_thread(phases, Duration::from_secs(2), db_pool.clone());

    let bind_address = "0.0.0.0:8080";
    info!("Starting server on '{}'", bind_address);
    let bind_result = HttpServer::new(move || {
        App::new()
            .data(db_pool.clone())
            .wrap(middleware::Logger::default())
            .app_data(
                web::JsonConfig::default()
                    .limit(4096)
                    .error_handler(handlers::json_error_handler),
            )
            .service(get_user)
            .service(create_user)
            .service(add_reminder)
            .default_service(web::to(HttpResponse::NotFound))
    })
    .bind(bind_address)
    .map_err(|_| StartError::FailedBind {
        address: bind_address.to_string(),
    })?;

    Ok(bind_result
        .run()
        .await
        .map_err(|_| StartError::RuntimeError)?)
}

#[post("/create_user")]
async fn create_user(
    pool: web::Data<DbPool>,
    request: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let user_id = web::block(move || db_actions::insert_user(&request.email, &conn))
        .await
        .map_err(|e| {
            error!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    Ok(HttpResponse::Ok().json(CreateUserResponse { user_id }))
}

#[post("/add_reminder")]
async fn add_reminder(
    pool: web::Data<DbPool>,
    request: web::Json<CreateMemoryRequest>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let memory_id = web::block(move || {
        db_actions::insert_reminder(
            request.user_id,
            request.topic.as_deref(),
            &request.text,
            &conn,
        )
    })
    .await
    .map_err(|e| {
        error!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;

    Ok(HttpResponse::Ok().json(CreateMemoryResponse { memory_id }))
}

#[get("/user/{user_id}")]
async fn get_user(
    pool: web::Data<DbPool>,
    user_id_param: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let user_id = user_id_param.into_inner();
    let user = web::block(move || db_actions::get_user(user_id, &conn))
        .await
        .map_err(|e| {
            error!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    let result = match user {
        None => HttpResponse::NotFound().body(format!("No user found with id '{}'", user_id)),
        Some(u) => HttpResponse::Ok().json(u),
    };

    Ok(result)
}

// ============================== tests ==============================
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::dev::Service;
    use actix_web::Error;
    use actix_web::{http, test, App};

    #[actix_rt::test]
    async fn test_index() -> Result<(), Error> {
        let mut app = test::init_service(App::new().service(create_user)).await;

        let req = test::TestRequest::post()
            .uri("/create_user")
            .set_json(&CreateUserRequest {
                email: "vasia@ya.ru".to_owned(),
            })
            .to_request();

        let resp = app.call(req).await.unwrap();

        let body = resp.response().body();
        match body {
            actix_web::dev::ResponseBody::Body(_some_b) => println!("some body"),
            actix_web::dev::ResponseBody::Other(other_b) => match other_b {
                actix_web::dev::Body::None => println!("body::none"),
                actix_web::dev::Body::Empty => println!("body::empty"),

                actix_web::dev::Body::Bytes(bytes) => {
                    let s = std::str::from_utf8(&bytes).expect("utf8 parse error)");
                    println!("html: {:?}", s)
                }
                actix_web::dev::Body::Message(_msg) => println!("body::msg"),
            },
        }

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = match resp.response().body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => bytes,
            _ => panic!("Response error"),
        };

        assert_eq!(response_body, r##"user 'some-name' created!"##);

        Ok(())
    }
}
