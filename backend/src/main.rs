use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8080;
    HttpServer::new(|| App::new().service(create_routine))
        .bind(("127.0.0.1", port))?
        .workers(2)
        .run()
        .await
}
#[derive(Deserialize, Serialize)]
pub struct Routine {
    name: String,
    exercise_list: Vec<Exercise>,
}

#[derive(Deserialize, Serialize)]
pub struct Exercise {
    name: String,
    exercise_type: ExerciseType,
}

pub enum ExerciseType {
    Single,
    Double,
}

#[actix_web::post("/routines/create/")]
async fn create_routine(routine_details: web::Path<(Routine)>) -> impl Responder {}
