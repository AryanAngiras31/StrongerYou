use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use serde::{de::value::Error, Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use std::env;
use std::fs;

//mod routines;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8080;
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let database_url = env::var("DATABASE_URL").expect("Failed to obtain database url");
    print!("database url : {}\n", database_url);
    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to form database pool");

    //Initialize database
    let sql_file = "init.sql"; // Path to your SQL file
    let sql_content = fs::read_to_string(sql_file).expect("Failed to read SQL file");

    // Execute the SQL commands
    sqlx::query(&sql_content)
        .execute(&pool)
        .await
        .expect("Failed to execute SQL file");

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(pool.clone()))
            .service(test)
        //.service(routines::create_routine)
        //.service(routines::modify_routine)
        //.service(routines::get_routine_history)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[actix_web::get("/routines")]
async fn test() -> impl Responder {
    format!("Ok")
}
