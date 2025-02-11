use actix_web::{delete, get, post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use log::error;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

// Data structures for request/response handling
#[derive(Serialize, Deserialize, Debug)]
struct Exercise {
    exercise_name: String,
    muscles_trained: Vec<String>,
    exercise_type: String,
}

#[derive(Serialize, Deserialize)]
struct TimeRange {
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct ExerciseStats {
    date: DateTime<Utc>,
    value: f64,
}

#[derive(Serialize, Deserialize)]
struct PersonalRecord {
    workout_date: DateTime<Utc>,
    weight: i16,
    reps: i16,
    one_rm: f32,
    set_volume: i32,
}

#[derive(sqlx::FromRow, Serialize)]
struct ExerciseSearchResult {
    exerciseid: i32,
    exercisename: String,
    muscles_trained: Vec<String>,
}

// Custom error response structure
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    details: Option<String>,
}

// Search for exercises by name
#[get("/exercises/{exercise_name}")]
async fn search_exercise(
    pool: web::Data<PgPool>,
    exercise_name: web::Path<String>,
) -> impl Responder {
    let exercises = sqlx::query_as!(
        ExerciseSearchResult,
        r#"
        SELECT ExerciseID as exerciseid, ExerciseName as exercisename, MusclesTrained as muscles_trained
        FROM ExerciseList 
        WHERE ExerciseName ILIKE $1
        "#,
        format!("%{}%", exercise_name.as_ref())
    )
    .fetch_all(pool.get_ref())
    .await;

    match exercises {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(e) => {
            error!("Database error in search_exercise: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to search exercises".to_string(),
                details: Some(e.to_string()),
            })
        }
    }
}

// Create a new exercise
#[post("/exercises")]
async fn create_exercise(pool: web::Data<PgPool>, exercise: web::Json<Exercise>) -> impl Responder {
    error!("Received exercise data: {:?}", exercise); // Log the received data

    let existing = sqlx::query!(
        "SELECT ExerciseID FROM ExerciseList WHERE ExerciseName = $1",
        exercise.exercise_name
    )
    .fetch_optional(pool.get_ref())
    .await;

    match existing {
        Ok(Some(_)) => HttpResponse::Conflict().json(ErrorResponse {
            error: "Exercise already exists".to_string(),
            details: None,
        }),
        Ok(None) => {
            let result = sqlx::query!(
                r#"
                INSERT INTO ExerciseList (ExerciseName, MusclesTrained)
                VALUES ($1, $2)
                RETURNING ExerciseID, ExerciseName, MusclesTrained
                "#,
                exercise.exercise_name,
                &exercise.muscles_trained as &[String] // Explicit type annotation
            )
            .fetch_one(pool.get_ref())
            .await;

            match result {
                Ok(row) => HttpResponse::Created().json(row),
                Err(e) => {
                    error!("Database error in create_exercise: {:?}", e);
                    HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "Failed to create exercise".to_string(),
                        details: Some(format!("Database error: {}", e)),
                    })
                }
            }
        }
        Err(e) => {
            error!("Database error checking existing exercise: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Database error".to_string(),
                details: Some(e.to_string()),
            })
        }
    }
}

// Delete an exercise
#[delete("/exercises/{exercise_name}")]
async fn delete_exercise(
    pool: web::Data<PgPool>,
    exercise_name: web::Path<String>,
) -> impl Responder {
    let result = sqlx::query!(
        "DELETE FROM ExerciseList WHERE ExerciseName = $1 RETURNING ExerciseID",
        exercise_name.as_ref()
    )
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(row)) => HttpResponse::Ok().json(row),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse {
            error: "Exercise not found".to_string(),
            details: None,
        }),
        Err(e) => {
            error!("Database error in delete_exercise: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to delete exercise".to_string(),
                details: Some(e.to_string()),
            })
        }
    }
}

// Get set volume history for an exercise
#[get("/exercises/{exercise_id}/volume")]
async fn get_exercise_volume(
    pool: web::Data<PgPool>,
    exercise_id: web::Path<i16>,
) -> impl Responder {
    let volumes = sqlx::query!(
        r#"
        SELECT w.Start as workout_date, SUM(s.Weight * s.Reps) as volume
        FROM Workout w
        JOIN Workout_Exercises_Sets wes ON w.WorkoutID = wes.WorkoutID
        JOIN "Set" s ON wes.SetID = s.SetID
        WHERE wes.ExerciseID = $1 
        GROUP BY w.Start
        ORDER BY w.Start
        "#,
        exercise_id.into_inner()
    )
    .fetch_all(pool.get_ref())
    .await;

    match volumes {
        Ok(results) => {
            let stats: Vec<ExerciseStats> = results
                .into_iter()
                .map(|row| ExerciseStats {
                    date: DateTime::from_naive_utc_and_offset(row.workout_date, Utc),
                    value: row.volume.unwrap_or(0) as f64,
                })
                .collect();
            HttpResponse::Ok().json(stats)
        }
        Err(e) => {
            error!("Database error in get_exercise_volume: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch volume data".to_string(),
                details: Some(e.to_string()),
            })
        }
    }
}

// Get max weight history for an exercise
#[get("/exercises/{exercise_id}/max-weight")]
async fn get_exercise_max_weight(
    pool: web::Data<PgPool>,
    exercise_id: web::Path<i16>,
) -> impl Responder {
    let max_weights = sqlx::query!(
        r#"
        SELECT w.Start as workout_date, MAX(s.Weight) as max_weight
        FROM Workout w
        JOIN Workout_Exercises_Sets wes ON w.WorkoutID = wes.WorkoutID
        JOIN "Set" s ON wes.SetID = s.SetID
        WHERE wes.ExerciseID = $1 
        GROUP BY w.Start
        ORDER BY w.Start
        "#,
        exercise_id.into_inner()
    )
    .fetch_all(pool.get_ref())
    .await;

    match max_weights {
        Ok(results) => {
            let stats: Vec<ExerciseStats> = results
                .into_iter()
                .map(|row| ExerciseStats {
                    date: DateTime::from_naive_utc_and_offset(row.workout_date, Utc),
                    value: row.max_weight.unwrap_or(0) as f64,
                })
                .collect();
            HttpResponse::Ok().json(stats)
        }
        Err(e) => {
            error!("Database error in get_exercise_max_weight: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch max weight data".to_string(),
                details: Some(e.to_string()),
            })
        }
    }
}

// Get PRs for an exercise
#[get("/exercises/{exercise_id}/prs")]
async fn get_exercise_prs(pool: web::Data<PgPool>, exercise_id: web::Path<i16>) -> impl Responder {
    let prs = sqlx::query!(
        r#"
        SELECT w.Start as workout_date, p.HeaviestWeight, p.OneRM as one_rm, p.SetVolume
        FROM PRs p
        JOIN Workout w ON p.WorkoutID = w.WorkoutID
        WHERE p.ExerciseID = $1
        ORDER BY p.OneRM DESC, p.HeaviestWeight DESC, p.SetVolume DESC
        "#,
        exercise_id.into_inner()
    )
    .fetch_all(pool.get_ref())
    .await;

    match prs {
        Ok(results) => {
            let pr_records: Vec<PersonalRecord> = results
                .into_iter()
                .map(|row| PersonalRecord {
                    workout_date: DateTime::from_naive_utc_and_offset(row.workout_date, Utc),
                    weight: row.heaviestweight,
                    one_rm: row.one_rm,
                    set_volume: row.setvolume,
                    reps: 0,
                })
                .collect();
            HttpResponse::Ok().json(pr_records)
        }
        Err(e) => {
            error!("Database error in get_exercise_prs: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch PRs".to_string(),
                details: Some(e.to_string()),
            })
        }
    }
}

// Initialize all routes
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(search_exercise)
        .service(create_exercise)
        .service(delete_exercise)
        .service(get_exercise_volume)
        .service(get_exercise_max_weight)
        .service(get_exercise_prs);
}
