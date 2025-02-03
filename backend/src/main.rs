use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use serde::{de::value::Error, Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use std::env;

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
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS routines (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to initialize tables in the database");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS exercises (
            id SERIAL PRIMARY KEY,
            routine_id INTEGER REFERENCES routines(id),
            name VARCHAR NOT NULL,
            exercise_type VARCHAR NOT NULL,
            number_of_sets INTEGER NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to initialize tables in the database");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS routine_history (
            id SERIAL PRIMARY KEY,
            routine_id INTEGER REFERENCES routines(id),
            completed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to initialize tables in the database");

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
