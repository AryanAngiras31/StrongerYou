use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use env_logger::Env;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fs;

#[derive(Serialize, Deserialize)]
struct Exercise {
    exercise_name: String,
    muscles_trained: Vec<String>,
    exercise_type: String,
}

#[post("/exercises")]
async fn add_exercise(pool: web::Data<PgPool>, req: web::Json<Exercise>) -> impl Responder {
    let result = sqlx::query!(
        "INSERT INTO ExerciseList (ExerciseName, MusclesTrained, ExerciseType) VALUES ($1, $2, $3)",
        req.exercise_name,
        &req.muscles_trained,
        req.exercise_type
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json("Exercise added successfully"),
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}

#[get("/highest_weights/{exercise_id}/{date_range}")]
async fn get_highest_weights(
    pool: web::Data<PgPool>,
    path: web::Path<(i32, String)>,
) -> impl Responder {
    let (exercise_id, date_range) = path.into_inner();
    let highest_weights = sqlx::query!(
        "SELECT w.WorkoutID, MAX(s.Weight) as HighestWeight
        FROM Workout_Exercises_Sets wes
        JOIN "Set" s ON wes.SetID = s.SetID
        JOIN Workout w ON wes.WorkoutID = w.WorkoutID
        WHERE wes.ExerciseID = $1 AND w.Start >= NOW() - INTERVAL $2
        GROUP BY w.WorkoutID",
        exercise_id,
        date_range
    )
    .fetch_all(pool.get_ref())
    .await;

    match highest_weights {
        Ok(weights) => HttpResponse::Ok().json(weights),
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}

#[get("/prs/{exercise_id}")]
async fn get_exercise_prs(pool: web::Data<PgPool>, path: web::Path<i32>) -> impl Responder {
    let exercise_id = path.into_inner();
    let prs = sqlx::query!("SELECT * FROM PRs WHERE ExerciseID = $1", exercise_id)
        .fetch_all(pool.get_ref())
        .await;

    match prs {
        Ok(prs) => HttpResponse::Ok().json(prs),
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}
