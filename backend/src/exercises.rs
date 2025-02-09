use actix_web::{delete, get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Deserialize, Serialize)]
struct Exercise {
    exercise_name: String,
    muscles_trained: Vec<String>,
    exercise_type: String,
}

#[derive(Serialize, Deserialize)]
struct Record {
    exercise_name: String,
    muscles_trained: Vec<String>,
    exercise_type: String,
}

#[get("/exercises/{exercise_name}")]
async fn search_exercise(
    pool: web::Data<PgPool>,
    exercise_name: web::Path<String>,
) -> impl Responder {
    let exercise = sqlx::query!(
        "SELECT * FROM ExerciseList WHERE ExerciseName = $1",
        exercise_name.into_inner()
    )
    .fetch_optional(pool.get_ref())
    .await;

    match exercise {
        Ok(Some(record)) => HttpResponse::Ok().json(record),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/exercises")]
async fn create_exercise(pool: web::Data<PgPool>, req: web::Json<Exercise>) -> impl Responder {
    let result = sqlx::query!(
        "INSERT INTO ExerciseList (ExerciseName, MusclesTrained, ExerciseType) VALUES ($1, $2, $3)",
        req.exercise_name,
        &req.muscles_trained,
        req.exercise_type
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/exercises/{exercise_name}")]
async fn delete_exercise(
    pool: web::Data<PgPool>,
    exercise_name: web::Path<String>,
) -> impl Responder {
    let result = sqlx::query!(
        "DELETE FROM ExerciseList WHERE ExerciseName = $1",
        exercise_name.into_inner()
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => HttpResponse::Ok().finish(),
        _ => HttpResponse::NotFound().finish(),
    }
}

#[get("/exercises/{exercise_id}/highest_weight/{range}")]
async fn highest_weight_per_workout(
    pool: web::Data<PgPool>,
    exercise_id: web::Path<i32>,
    range: web::Path<String>,
) -> impl Responder {
    let highest_weights = sqlx::query!(
        r#"
        SELECT w.WorkoutID, MAX(s.Weight) as HighestWeight
        FROM Workout_Exercises_Sets wes
        JOIN "Set" s ON wes.SetID = s.SetID
        JOIN Workout w ON wes.WorkoutID = w.WorkoutID
        WHERE wes.ExerciseID = $1 AND w.Start >= NOW() - INTERVAL $2
        GROUP BY w.WorkoutID
        "#,
        exercise_id,
        date_range
    )
    .fetch_all(pool.get_ref())
    .await?;

    match weights {
        Ok(records) => HttpResponse::Ok().json(records),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/exercises/{exercise_id}/set_volume/{range}")]
async fn set_volume_per_workout(
    pool: web::Data<PgPool>,
    exercise_id: web::Path<i32>,
    range: web::Path<String>,
) -> impl Responder {
    let volumes = sqlx::query!(
        "SELECT w.WorkoutID, SUM(s.Weight * s.Reps) AS TotalVolume
         FROM Workout_Exercises_Sets wes
         JOIN "Set" s ON wes.SetID = s.SetID
         JOIN Workout w ON wes.WorkoutID = w.WorkoutID
         WHERE wes.ExerciseID = $1 AND w.Start >= NOW() - INTERVAL $2
         GROUP BY w.WorkoutID",
        exercise_id.into_inner(),
        range.into_inner()
    )
    .fetch_all(pool.get_ref())
    .await;

    match volumes {
        Ok(records) => HttpResponse::Ok().json(records),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/exercises/{exercise_id}/prs")]
async fn get_exercise_prs(pool: web::Data<PgPool>, exercise_id: web::Path<i32>) -> impl Responder {
    let prs = sqlx::query!("SELECT * FROM PRs WHERE ExerciseID = $1", exercise_id)
        .fetch_all(pool.get_ref())
        .await?;

    match prs {
        Ok(records) => HttpResponse::Ok().json(records),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
