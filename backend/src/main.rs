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
    let sql_content_vec = parse_sql_file(&sql_file);
    print!("\nsql_content_vec :\n{:#?}", sql_content_vec);
    for statement in sql_content_vec {
        // Execute the SQL commands
        sqlx::query(&statement.unwrap())
            .execute(&pool)
            .await
            .expect("Failed to execute SQL file");
    }

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

fn parse_sql_file(file_path: &str) -> Result<Vec<String>, std::io::Error> {
    // Read the file content
    let content = fs::read_to_string(file_path)?;

    let mut statements = Vec::new();
    let mut current_statement = String::new();
    let mut in_multiline_comment = false;

    // Process the file line by line
    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Handle multiline comments
        if trimmed.starts_with("/*") {
            in_multiline_comment = true;
            continue;
        }
        if trimmed.ends_with("*/") {
            in_multiline_comment = false;
            continue;
        }
        if in_multiline_comment {
            continue;
        }

        // Skip single-line comments
        if trimmed.starts_with("--") {
            continue;
        }

        // Add the line to the current statement
        current_statement.push_str(trimmed);

        // If the line ends with a semicolon, it's the end of a statement
        if trimmed.ends_with(";") {
            // Clean up the statement and add it to the vector
            let clean_statement = current_statement.trim().to_string();
            if !clean_statement.is_empty() {
                statements.push(clean_statement);
            }
            current_statement.clear();
        } else {
            // Add a space between lines for multiline statements
            current_statement.push(' ');
        }
    }

    Ok(statements)
}
