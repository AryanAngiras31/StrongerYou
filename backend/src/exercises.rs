use actix_web::{delete, get, post, web, HttpResponse, Responder};
// Make sure NaiveDateTime is imported if you use it directly
use chrono::{DateTime, NaiveDateTime, Utc};
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

// Data structures for request/response handling
#[derive(Serialize, Deserialize, Debug)]
struct ExerciseInput {
    exercise_name: String,
    muscles_trained: Vec<String>,
    exercise_type: String,
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

#[derive(sqlx::FromRow, Serialize)]
struct ExerciseIdResult {
    exerciseid: i32,
}

#[derive(sqlx::FromRow, Serialize)]
struct ExerciseDetails {
    exerciseid: i32,
    exercisename: String,
    muscles_trained: Vec<String>,
    exercisetype: String,
}

#[derive(sqlx::FromRow, Serialize)]
struct DeletedExercise {
    exerciseid: i32,
}

// Search for exercises by partial name (Unchanged)
#[get("/exercises/search/{partial_name}")]
async fn search_exercises_by_name(
    pool: web::Data<PgPool>,
    partial_name: web::Path<String>,
) -> impl Responder {
    let search_term = format!("%{}%", partial_name.as_ref());
    let exercises = sqlx::query_as!(
        ExerciseSearchResult,
        r#"
        SELECT
            ExerciseID as exerciseid,
            ExerciseName as exercisename,
            MusclesTrained as muscles_trained
        FROM ExerciseList
        WHERE ExerciseName ILIKE $1
        ORDER BY ExerciseName
        LIMIT 20
        "#,
        search_term
    )
    .fetch_all(pool.get_ref())
    .await;

    match exercises {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(e) => {
            error!("Database error in search_exercises_by_name: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to search exercises",
                "details": e.to_string()
            }))
        }
    }
}

// Get exercise ID by exact name (Unchanged)
#[get("/exercises/id/{exercise_name}")]
async fn get_exercise_id_by_name(
    pool: web::Data<PgPool>,
    exercise_name: web::Path<String>,
) -> impl Responder {
    let name = exercise_name.into_inner();
    let exercise_id_result = sqlx::query_as!(
        ExerciseIdResult,
        r#"
        SELECT ExerciseID as exerciseid
        FROM ExerciseList
        WHERE ExerciseName ILIKE $1
        "#,
        name
    )
    .fetch_optional(pool.get_ref())
    .await;

    match exercise_id_result {
        Ok(Some(result)) => HttpResponse::Ok().json(result),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "error": "Exercise not found",
            "details": format!("No exercise found with the exact name: {}", name)
        })),
        Err(e) => {
            error!("Database error in get_exercise_id_by_name: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch exercise ID by name",
                "details": e.to_string()
            }))
        }
    }
}

// Create a new exercise (Unchanged)
#[post("/exercises")]
async fn create_exercise(
    pool: web::Data<PgPool>,
    exercise_input: web::Json<ExerciseInput>,
) -> impl Responder {
    match sqlx::query!(
        "SELECT ExerciseID FROM ExerciseList WHERE ExerciseName ILIKE $1",
        exercise_input.exercise_name
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(_)) => {
            HttpResponse::Conflict().json(json!({
                "error": "Exercise already exists",
                "details": format!("An exercise with the name '{}' already exists (case-insensitive).", exercise_input.exercise_name)
            }))
        }
        Ok(None) => {
            let result = sqlx::query_as!(
                ExerciseDetails,
                r#"
                INSERT INTO ExerciseList (ExerciseName, MusclesTrained, ExerciseType)
                VALUES ($1, $2, $3)
                RETURNING
                    ExerciseID as exerciseid,
                    ExerciseName as exercisename,
                    MusclesTrained as muscles_trained,
                    ExerciseType as exercisetype
                "#,
                exercise_input.exercise_name,
                &exercise_input.muscles_trained,
                exercise_input.exercise_type
            )
            .fetch_one(pool.get_ref())
            .await;

            match result {
                Ok(created_exercise) => HttpResponse::Created().json(created_exercise),
                Err(e) => {
                    error!("Database error in create_exercise during INSERT: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "error": "Failed to create exercise",
                        "details": e.to_string()
                    }))
                }
            }
        }
        Err(e) => {
            error!("Database error in create_exercise checking existence: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Database error while checking for existing exercise",
                "details": e.to_string()
            }))
        }
    }
}

// Delete an exercise by ID
#[delete("/exercises/{exercise_id}")]
async fn delete_exercise(
    pool: web::Data<PgPool>,
    exercise_id: web::Path<i32>, // Keep i32 if ExerciseList.ExerciseID is INTEGER
) -> impl Responder {
    // Note: Assuming ExerciseList.ExerciseID is i32. If it's SMALLINT, change Path to i16.
    let id = exercise_id.into_inner();
    let result = sqlx::query_as!(
        DeletedExercise,
        r#"
        DELETE FROM ExerciseList
        WHERE ExerciseID = $1
        RETURNING ExerciseID as exerciseid
        "#,
        id // Pass i32 directly
    )
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(deleted_exercise)) => HttpResponse::Ok().json(deleted_exercise),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "error": "Exercise not found",
            "details": format!("Exercise with ID {} could not be deleted because it was not found.", id)
        })),
        Err(e) => {
            error!("Database error in delete_exercise: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to delete exercise",
                "details": e.to_string()
            }))
        }
    }
}

// Get set volume history for an exercise by ID
#[get("/exercises/volume/{exercise_id}")]
async fn get_exercise_volume(
    pool: web::Data<PgPool>,
    exercise_id: web::Path<i16>, // Keep i16 based on previous error
) -> impl Responder {
    // Define the expected return types - expecting i64 due to SQL CAST
    struct VolumeRow {
        workout_date_naive: Option<NaiveDateTime>,
        total_volume: Option<i64>, // Expecting BIGINT from SQL CAST
    }

    let volumes_raw = sqlx::query_as!(
        VolumeRow,
        r#"
        SELECT w.Start as workout_date_naive,
               -- Cast the final SUM result to BIGINT (i64) in SQL
               CAST( SUM( s.Weight * s.Reps ) AS BIGINT ) as total_volume
        FROM Workout w
        JOIN Workout_Exercises_Sets wes ON w.WorkoutID = wes.WorkoutID
        JOIN "Set" s ON wes.SetID = s.SetID
        WHERE wes.ExerciseID = $1 -- Expects i16 here
        GROUP BY w.Start
        ORDER BY w.Start
        "#,
        exercise_id.into_inner() // Provides i16
    )
    .fetch_all(pool.get_ref())
    .await;

    match volumes_raw {
        Ok(results) => {
            let stats: Vec<ExerciseStats> = results
                .into_iter()
                .filter_map(|row| {
                    // Match on the Option fields
                    match (row.workout_date_naive, row.total_volume) {
                        (Some(date_naive), Some(volume_numeric)) => {
                            // volume_numeric is i64
                            let date_utc = DateTime::from_naive_utc_and_offset(date_naive, Utc);
                            // Convert the i64 volume to f64
                            let value = volume_numeric as f64;
                            Some(ExerciseStats {
                                date: date_utc,
                                value,
                            })
                        }
                        _ => None, // Skip if date or volume is None
                    }
                })
                .collect();
            HttpResponse::Ok().json(stats)
        }
        Err(e) => {
            error!("Database error in get_exercise_volume: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch volume data",
                "details": e.to_string()
            }))
        }
    }
}

// Get max weight history for an exercise by ID
#[get("/exercises/max-weight/{exercise_id}")]
async fn get_exercise_max_weight(
    pool: web::Data<PgPool>,
    exercise_id: web::Path<i16>, // <<<<<<<< CHANGED to i16 based on error E0308
) -> impl Responder {
    // Define expected return types
    struct MaxWeightRow {
        workout_date_naive: Option<NaiveDateTime>,
        max_weight_val: Option<i16>, // Assuming MAX(s.Weight) returns SMALLINT
    }

    let max_weights_raw = sqlx::query_as!(
        MaxWeightRow,
        r#"
        SELECT w.Start as workout_date_naive, MAX(s.Weight) as max_weight_val
        FROM Workout w
        JOIN Workout_Exercises_Sets wes ON w.WorkoutID = wes.WorkoutID
        JOIN "Set" s ON wes.SetID = s.SetID
        WHERE wes.ExerciseID = $1 AND s.Weight IS NOT NULL -- Expects i16 here based on error E0308
        GROUP BY w.Start
        ORDER BY w.Start
        "#,
        exercise_id.into_inner() // Provides i16
    )
    .fetch_all(pool.get_ref())
    .await;

    match max_weights_raw {
        Ok(results) => {
            let stats: Vec<ExerciseStats> = results
                .into_iter()
                .filter_map(|row| {
                    // Match on the Option fields
                    match (row.workout_date_naive, row.max_weight_val) {
                        (Some(date_naive), Some(max_w)) => {
                            let date_utc = DateTime::from_naive_utc_and_offset(date_naive, Utc);
                            Some(ExerciseStats {
                                date: date_utc,
                                value: max_w as f64,
                            })
                        }
                        _ => None, // Skip if date or max_weight is None
                    }
                })
                .collect();
            HttpResponse::Ok().json(stats)
        }
        Err(e) => {
            error!("Database error in get_exercise_max_weight: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch max weight data",
                "details": e.to_string()
            }))
        }
    }
}

// Get PRs for an exercise by ID
#[get("/exercises/prs/{exercise_id}")]
async fn get_exercise_prs(
    pool: web::Data<PgPool>,
    exercise_id: web::Path<i16>, // <<<<<<<< CHANGED to i16 based on error E0308
) -> impl Responder {
    // Define expected return types, assuming columns in PRs can be NULL
    struct PrRow {
        workout_date_naive: Option<NaiveDateTime>,
        heaviest_weight: Option<i16>,
        one_rm: Option<f32>,
        set_volume: Option<i32>,
    }

    let prs_raw = sqlx::query_as!(
        PrRow,
        r#"
        SELECT
            w.Start as workout_date_naive,
            p.HeaviestWeight as heaviest_weight,
            p.OneRM as one_rm,
            p.SetVolume as set_volume
        FROM PRs p
        JOIN Workout w ON p.WorkoutID = w.WorkoutID
        WHERE p.ExerciseID = $1 -- Expects i16 here based on error E0308
        ORDER BY p.OneRM DESC, p.HeaviestWeight DESC, p.SetVolume DESC
        "#,
        exercise_id.into_inner() // Provides i16
    )
    .fetch_all(pool.get_ref())
    .await;

    match prs_raw {
        Ok(results) => {
            let pr_records: Vec<PersonalRecord> = results
                .into_iter()
                .filter_map(|row| {
                    // Match on all required Option fields from the row
                    match (
                        row.workout_date_naive,
                        row.heaviest_weight,
                        row.one_rm,
                        row.set_volume,
                    ) {
                        (Some(date_naive), Some(weight), Some(one_rm), Some(set_volume)) => {
                            let date_utc = DateTime::from_naive_utc_and_offset(date_naive, Utc);
                            Some(PersonalRecord {
                                workout_date: date_utc,
                                weight,     // Use the unwrapped value
                                one_rm,     // Use the unwrapped value
                                set_volume, // Use the unwrapped value
                                reps: 0,
                            })
                        }
                        // If any of the required fields are None, skip this record
                        _ => None,
                    }
                })
                .collect();
            HttpResponse::Ok().json(pr_records)
        }
        Err(e) => {
            error!("Database error in get_exercise_prs: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch PRs",
                "details": e.to_string()
            }))
        }
    }
}

// Initialize all routes (Unchanged)
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(search_exercises_by_name)
        .service(get_exercise_id_by_name)
        .service(create_exercise)
        .service(delete_exercise)
        .service(get_exercise_volume)
        .service(get_exercise_max_weight)
        .service(get_exercise_prs);
}
