#![allow(unused_imports)]
use actix_cors::Cors;
use actix_web::cookie::time::error;
use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::{http, middleware};
use dotenv::dotenv;
use env_logger::Env;
use log::{error, info, warn};
use serde::{de::value::Error, Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use std::env;
use std::fs;

mod db;
mod exercises;
mod markers;
mod routines;
mod workouts;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8081;
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Initialize the database
    let pool = match db::initialize_database().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            return Ok(()); // Or handle the error more severely
        }
    };

    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:8101") // Your frontend's origin
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
            .max_age(3600);
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .wrap(Logger::default()) // Enable logging
            .app_data(web::Data::new(pool.clone()))
            .configure(configure_routes)
    })
    .bind(("localhost", port))?
    .run()
    .await
}

fn configure_routes(cfg: &mut web::ServiceConfig) {
    exercises::init_routes(cfg);
    markers::init_routes(cfg);
    routines::init_routes(cfg);
    workouts::init_routes(cfg);
}
