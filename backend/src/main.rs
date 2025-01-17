use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};

mod routine;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8080;
    HttpServer::new(|| App::new().service(routine::create_routine))
        .bind(("127.0.0.1", port))?
        .workers(2)
        .run()
        .await
}
