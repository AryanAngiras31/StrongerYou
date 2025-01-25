use actix_web::{get, post, web, Error, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

// Data structures for our API
#[derive(Deserialize)]
struct Exercise {
    exercise_name: String,
    exercise_type: String,
    number_of_sets: i32,
}

#[derive(Deserialize)]
struct CreateRoutineRequest {
    routine_name: String,
    exercise_list: Vec<Exercise>,
}

#[derive(Deserialize)]
struct ModifyRoutineRequest {
    routine_id: i32,
    routine_name: String,
    exercise_list: Vec<Exercise>,
}

#[derive(Serialize)]
struct RoutineHistory {
    routine_id: i32,
    routine_name: String,
    completed_at: DateTime<Utc>,
}

// Create a new routine
#[post("/routines")]
async fn create_routine(
    pool: web::Data<PgPool>,
    request: web::Json<CreateRoutineRequest>,
) -> Result<HttpResponse, Error> {
    // Start a transaction
    let mut tx = pool.begin().await?;

    // Insert the routine
    let routine_id = sqlx::query!(
        "INSERT INTO routines (name) VALUES ($1) RETURNING id",
        request.routine_name
    )
    .fetch_one(&mut tx)
    .await?
    .id;

    // Insert each exercise
    for exercise in &request.exercise_list {
        sqlx::query!(
            "INSERT INTO exercises (routine_id, name, exercise_type, number_of_sets) 
             VALUES ($1, $2, $3, $4)",
            routine_id,
            exercise.exercise_name,
            exercise.exercise_type,
            exercise.number_of_sets
        )
        .execute(&mut tx)
        .await?;
    }

    // Commit the transaction
    tx.commit().await?;

    Ok(HttpResponse::Created().json(routine_id))
}

// Modify an existing routine
#[post("/routines/{id}")]
async fn modify_routine(
    pool: web::Data<PgPool>,
    request: web::Json<ModifyRoutineRequest>,
) -> Result<HttpResponse, Error> {
    // Start a transaction
    let mut tx = pool.begin().await?;

    // Update routine name
    sqlx::query!(
        "UPDATE routines SET name = $1 WHERE id = $2",
        request.routine_name,
        request.routine_id
    )
    .execute(&mut tx)
    .await?;

    // Delete existing exercises
    sqlx::query!(
        "DELETE FROM exercises WHERE routine_id = $1",
        request.routine_id
    )
    .execute(&mut tx)
    .await?;

    // Insert new exercises
    for exercise in &request.exercise_list {
        sqlx::query!(
            "INSERT INTO exercises (routine_id, name, exercise_type, number_of_sets) 
             VALUES ($1, $2, $3, $4)",
            request.routine_id,
            exercise.exercise_name,
            exercise.exercise_type,
            exercise.number_of_sets
        )
        .execute(&mut tx)
        .await?;
    }

    // Record this modification in history
    sqlx::query!(
        "INSERT INTO routine_history (routine_id) VALUES ($1)",
        request.routine_id
    )
    .execute(&mut tx)
    .await?;

    // Commit the transaction
    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

// Get routine history
#[get("/routines/history")]
async fn get_routine_history(
    pool: web::Data<PgPool>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
    // Determine sort order from query parameter
    let order = query.get("order").map_or("DESC", |o| {
        if o.to_lowercase() == "asc" {
            "ASC"
        } else {
            "DESC"
        }
    });

    // Query routine history with routine names
    let history = sqlx::query_as!(
        RoutineHistory,
        r#"
        SELECT 
            rh.routine_id,
            r.name as routine_name,
            rh.completed_at
        FROM routine_history rh
        JOIN routines r ON r.id = rh.routine_id
        ORDER BY rh.completed_at {}"#,
        order
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(history))
}
