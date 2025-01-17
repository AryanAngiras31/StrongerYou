use actix_web::{get, post, web, App, Error, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};

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

#[derive(Deserialize, Serialize)]
pub enum ExerciseType {
    Single,
    Double,
}

#[actix_web::post("/routines/create/")]
async fn create_routine(routine_details: web::Path<Routine>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}
