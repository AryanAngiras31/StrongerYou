use actix_web::{delete, get, post, web, HttpResponse, Responder};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

// Data structures for request/response handling
#[derive(Serialize, Deserialize)]
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

// Search for exercises by name
#[get("/exercises/{exercise_name}")]
async fn search_exercise(
    pool: web::Data<PgPool>,
    exercise_name: web::Path<String>,
) -> impl Responder {
    // Use ILIKE for case-insensitive search with pattern matching
    let exercises = sqlx::query!(
        r#"
        SELECT ExerciseID, ExerciseName, MusclesTrained 
        FROM ExerciseList 
        WHERE ExerciseName ILIKE $1
        "#,
        format!("%{}%", exercise_name.as_ref())
    )
    .fetch_all(pool.get_ref())
    .await;

    match exercises {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(_) => HttpResponse::InternalServerError().json("Failed to search exercises"),
    }
}

// Create a new exercise
#[post("/exercises")]
async fn create_exercise(pool: web::Data<PgPool>, exercise: web::Json<Exercise>) -> impl Responder {
    // Check if exercise with same name already exists
    let existing = sqlx::query!(
        "SELECT ExerciseID FROM ExerciseList WHERE ExerciseName = $1",
        exercise.exercise_name
    )
    .fetch_optional(pool.get_ref())
    .await;

    match existing {
        Ok(Some(_)) => HttpResponse::Conflict().json("Exercise already exists"),
        Ok(None) => {
            // Insert new exercise
            let result = sqlx::query!(
                r#"
                INSERT INTO ExerciseList (ExerciseName, MusclesTrained)
                VALUES ($1, $2)
                RETURNING ExerciseID
                "#,
                exercise.exercise_name,
                &exercise.muscles_trained
            )
            .fetch_one(pool.get_ref())
            .await;

            match result {
                Ok(row) => HttpResponse::Created().json(row.exerciseid),
                Err(_) => HttpResponse::InternalServerError().json("Failed to create exercise"),
            }
        }
        Err(_) => HttpResponse::InternalServerError().json("Database error"),
    }
}

// Delete an exercise
#[delete("/exercises/{exercise_name}")]
async fn delete_exercise(
    pool: web::Data<PgPool>,
    exercise_name: web::Path<String>,
) -> impl Responder {
    // Delete exercise if it exists
    let result = sqlx::query!(
        "DELETE FROM ExerciseList WHERE ExerciseName = $1 RETURNING ExerciseID",
        exercise_name.as_ref()
    )
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(_)) => HttpResponse::Ok().json("Exercise deleted successfully"),
        Ok(None) => HttpResponse::NotFound().json("Exercise not found"),
        Err(_) => HttpResponse::InternalServerError().json("Failed to delete exercise"),
    }
}

// Get set volume history for an exercise
#[get("/exercises/{exercise_id}/volume")]
async fn get_exercise_volume(
    pool: web::Data<PgPool>,
    exercise_id: web::Path<i32>,
    range: web::Query<TimeRange>,
) -> impl Responder {
    let volumes = sqlx::query!(
        r#"
        SELECT w.Start as workout_date, SUM(s.Weight * s.Reps) as volume
        FROM Workout w
        JOIN Workout_Exercises_Sets wes ON w.WorkoutID = wes.WorkoutID
        JOIN "Set" s ON wes.SetID = s.SetID
        WHERE wes.ExerciseID = $1 
        AND w.Start BETWEEN $2 AND $3
        GROUP BY w.Start
        ORDER BY w.Start
        "#,
        exercise_id.into_inner(),
        range.start_date.naive_utc(),
        range.end_date.naive_utc()
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
        Err(_) => HttpResponse::InternalServerError().json("Failed to fetch volume data"),
    }
}

// Get max weight history for an exercise
#[get("/exercises/{exercise_id}/max-weight")]
async fn get_exercise_max_weight(
    pool: web::Data<PgPool>,
    exercise_id: web::Path<i32>,
    range: web::Query<TimeRange>,
) -> impl Responder {
    let max_weights = sqlx::query!(
        r#"
        SELECT w.Start as workout_date, MAX(s.Weight) as max_weight
        FROM Workout w
        JOIN Workout_Exercises_Sets wes ON w.WorkoutID = wes.WorkoutID
        JOIN "Set" s ON wes.SetID = s.SetID
        WHERE wes.ExerciseID = $1 
        AND w.Start BETWEEN $2 AND $3
        GROUP BY w.Start
        ORDER BY w.Start
        "#,
        exercise_id.into_inner(),
        range.start_date.naive_utc(),
        range.end_date.naive_utc()
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
        Err(_) => HttpResponse::InternalServerError().json("Failed to fetch max weight data"),
    }
}

// Get PRs for an exercise
#[get("/exercises/{exercise_id}/prs")]
async fn get_exercise_prs(pool: web::Data<PgPool>, exercise_id: web::Path<i32>) -> impl Responder {
    let prs = sqlx::query!(
        r#"
        SELECT w.Start as workout_date, p.HeaviestWeight, p."1RM", p.SetVolume
        FROM PRs p
        JOIN Workout w ON p.WorkoutID = w.WorkoutID
        WHERE p.ExerciseID = $1
        ORDER BY p."1RM" DESC, p.HeaviestWeight DESC, p.SetVolume DESC
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
                    one_rm: row._1rm,
                    set_volume: row.setvolume,
                    reps: 0, // You might want to add this to your PRs table
                })
                .collect();
            HttpResponse::Ok().json(pr_records)
        }
        Err(_) => HttpResponse::InternalServerError().json("Failed to fetch PRs"),
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
