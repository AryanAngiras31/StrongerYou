use actix_web::{delete, get, post, put, web, HttpResponse, Scope};
use chrono::{NaiveDate, NaiveDateTime, Utc};
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, PgPool, Postgres, Row, Transaction};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct RoutineCreate {
    name: String,
    description: Option<String>,
    exercises: Vec<RoutineExercise>,
}

#[derive(Serialize, Deserialize)]
struct RoutineUpdate {
    name: String,
    description: Option<String>,
    exercises: Vec<RoutineExercise>,
}

#[derive(Serialize, Deserialize)]
struct RoutineExercise {
    exercise_id: i32,
    sets: i32,
}

#[derive(Serialize, FromRow)]
struct RoutineInfo {
    routine_id: i32,
    name: String,
    description: Option<String>,
    created_at: NaiveDateTime,
    last_performed: Option<NaiveDate>,
}

#[derive(Serialize)]
struct RoutineDetail {
    routine_id: i32,
    name: String,
    description: Option<String>,
    created_at: NaiveDateTime,
    exercises: Vec<RoutineExerciseDetail>,
}

#[derive(Serialize, FromRow)]
struct RoutineExerciseDetail {
    exercise_id: i32,
    exercise_name: String,
    sets: i32,
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_routine_by_name)
        .service(create_routine)
        .service(update_routine)
        .service(delete_routine)
        .service(list_routines);
}

#[get("/routines")]
async fn list_routines(
    pool: web::Data<PgPool>,
    request: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    // Verify sort parameter is by createdAt
    if request.get("sort") != Some(&"createdAt".to_string()) {
        return HttpResponse::BadRequest().json(json!({
            "error": "sort parameter must be 'createdAt'"
        }));
    }

    // Check if we need to include lastPerformed
    let include_last_performed = request.get("include") == Some(&"lastPerformed".to_string());

    // Query to get routines with optional last performed date
    let query = if include_last_performed {
        "SELECT r.RoutineID, r.RoutineName, r.Description, r.CreatedAt, 
         (SELECT MAX(w.Start::date) FROM Workout w WHERE w.RoutineID = r.RoutineID) as last_performed
         FROM Routines r ORDER BY r.CreatedAt ASC"
    } else {
        "SELECT r.RoutineID, r.RoutineName, r.Description, r.CreatedAt,
         NULL as last_performed
         FROM Routines r ORDER BY r.CreatedAt ASC"
    };

    match sqlx::query(query).fetch_all(pool.get_ref()).await {
        Ok(rows) => {
            let routines: Vec<RoutineInfo> = rows
                .iter()
                .map(|row| RoutineInfo {
                    routine_id: row.get("routineid"),
                    name: row.get("routinename"),
                    description: row.get("description"),
                    created_at: row.get("createdat"),
                    last_performed: row.get("last_performed"),
                })
                .collect();

            info!("Retrieved {} routines", routines.len());
            HttpResponse::Ok().json(routines)
        }
        Err(e) => {
            error!("Failed to fetch routines: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch routines"
            }))
        }
    }
}

#[get("/routines")]
async fn get_routine_by_name(
    pool: web::Data<PgPool>,
    request: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    // Otherwise proceed with getting routine by name
    let routine_name = match request.get("name") {
        Some(name) => name,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "name parameter is required"
            }))
        }
    };

    match sqlx::query("SELECT RoutineID FROM Routines WHERE RoutineName = $1")
        .bind(routine_name)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(row) => {
            let routine_id: i32 = row.get("routineid");
            info!(
                "Retrieved RoutineID {} for name {}",
                routine_id, routine_name
            );
            HttpResponse::Ok().json(json!({ "routine_id": routine_id }))
        }
        Err(e) => {
            error!("Failed to fetch routine ID: {}", e);
            HttpResponse::NotFound().json(json!({
                "error": format!("Routine with name '{}' not found", routine_name)
            }))
        }
    }
}

#[post("/routines")]
async fn create_routine(
    pool: web::Data<PgPool>,
    routine: web::Json<RoutineCreate>,
) -> HttpResponse {
    if routine.name.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "error": "Routine name cannot be empty"
        }));
    }

    if routine.exercises.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "error": "Routine must include at least one exercise"
        }));
    }

    // Start a transaction
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!("Failed to start transaction: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to create routine: database error"
            }));
        }
    };

    // Insert into Routines table
    let routine_id = match sqlx::query(
        "INSERT INTO Routines (RoutineName, Description, CreatedAt) 
         VALUES ($1, $2, $3) RETURNING RoutineID",
    )
    .bind(&routine.name)
    .bind(&routine.description)
    .bind(Utc::now().naive_utc())
    .fetch_one(&mut *tx) // Deref tx with &mut *tx
    .await
    {
        Ok(row) => row.get::<i32, _>("routineid"),
        Err(e) => {
            error!("Failed to insert into Routines table: {}", e);
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to create routine: {}", e)
            }));
        }
    };

    // Insert exercises and sets
    for exercise in &routine.exercises {
        if let Err(e) = sqlx::query(
            "INSERT INTO Routines_Exercises_Sets (RoutineID, ExerciseID, Sets) 
             VALUES ($1, $2, $3)",
        )
        .bind(routine_id)
        .bind(exercise.exercise_id)
        .bind(exercise.sets)
        .execute(&mut *tx) // Deref tx with &mut *tx
        .await
        {
            error!(
                "Failed to insert exercise {} into routine: {}",
                exercise.exercise_id, e
            );
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to add exercise to routine: {}", e)
            }));
        }
    }

    // Commit the transaction
    if let Err(e) = tx.commit().await {
        error!("Failed to commit transaction: {}", e);
        return HttpResponse::InternalServerError().json(json!({
            "error": "Failed to complete routine creation"
        }));
    }

    info!(
        "Created new routine: {} with ID {}",
        routine.name, routine_id
    );
    HttpResponse::Created().json(json!({ "routine_id": routine_id }))
}

#[put("/routines/{routine_id}")]
async fn update_routine(
    pool: web::Data<PgPool>,
    routine_id: web::Path<i32>,
    update: web::Json<RoutineUpdate>,
) -> HttpResponse {
    let routine_id = routine_id.into_inner();

    if update.name.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "error": "Routine name cannot be empty"
        }));
    }

    if update.exercises.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "error": "Routine must include at least one exercise"
        }));
    }

    // Check if routine exists
    match sqlx::query("SELECT RoutineID FROM Routines WHERE RoutineID = $1")
        .bind(routine_id)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(_) => (),
        Err(e) => {
            error!("Routine {} not found: {}", routine_id, e);
            return HttpResponse::NotFound().json(json!({
                "error": format!("Routine with ID {} not found", routine_id)
            }));
        }
    }

    // Start a transaction
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!("Failed to start transaction: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to update routine: database error"
            }));
        }
    };

    // Update Routines table
    if let Err(e) =
        sqlx::query("UPDATE Routines SET RoutineName = $1, Description = $2 WHERE RoutineID = $3")
            .bind(&update.name)
            .bind(&update.description)
            .bind(routine_id)
            .execute(&mut *tx) // Deref tx with &mut *tx
            .await
    {
        error!("Failed to update Routines table: {}", e);
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to update routine: {}", e)
        }));
    }

    // Delete old exercise associations
    if let Err(e) = sqlx::query("DELETE FROM Routines_Exercises_Sets WHERE RoutineID = $1")
        .bind(routine_id)
        .execute(&mut *tx) // Deref tx with &mut *tx
        .await
    {
        error!("Failed to delete old exercises: {}", e);
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().json(json!({
            "error": "Failed to update routine exercises"
        }));
    }

    // Insert new exercises and sets
    for exercise in &update.exercises {
        if let Err(e) = sqlx::query(
            "INSERT INTO Routines_Exercises_Sets (RoutineID, ExerciseID, Sets) 
             VALUES ($1, $2, $3)",
        )
        .bind(routine_id)
        .bind(exercise.exercise_id)
        .bind(exercise.sets)
        .execute(&mut *tx) // Deref tx with &mut *tx
        .await
        {
            error!(
                "Failed to insert exercise {} into routine: {}",
                exercise.exercise_id, e
            );
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to add exercise to routine: {}", e)
            }));
        }
    }

    // Commit the transaction
    if let Err(e) = tx.commit().await {
        error!("Failed to commit transaction: {}", e);
        return HttpResponse::InternalServerError().json(json!({
            "error": "Failed to complete routine update"
        }));
    }

    info!("Updated routine {}", routine_id);
    HttpResponse::Ok().json(json!({ "status": "updated" }))
}

#[delete("/routines/{routine_id}")]
async fn delete_routine(pool: web::Data<PgPool>, routine_id: web::Path<i32>) -> HttpResponse {
    let routine_id = routine_id.into_inner();

    // Start a transaction
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!("Failed to start transaction: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to delete routine: database error"
            }));
        }
    };

    // Delete from Routines_Exercises_Sets first
    if let Err(e) = sqlx::query("DELETE FROM Routines_Exercises_Sets WHERE RoutineID = $1")
        .bind(routine_id)
        .execute(&mut *tx) // Deref tx with &mut *tx
        .await
    {
        error!("Failed to delete from Routines_Exercises_Sets: {}", e);
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().json(json!({
            "error": "Failed to delete routine exercise associations"
        }));
    }

    // Then delete from Routines
    match sqlx::query("DELETE FROM Routines WHERE RoutineID = $1")
        .bind(routine_id)
        .execute(&mut *tx) // Deref tx with &mut *tx
        .await
    {
        Ok(result) => {
            if result.rows_affected() == 0 {
                let _ = tx.rollback().await;
                return HttpResponse::NotFound().json(json!({
                    "error": format!("Routine with ID {} not found", routine_id)
                }));
            }
        }
        Err(e) => {
            error!("Failed to delete from Routines: {}", e);
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to delete routine"
            }));
        }
    }

    // Commit the transaction
    if let Err(e) = tx.commit().await {
        error!("Failed to commit transaction: {}", e);
        return HttpResponse::InternalServerError().json(json!({
            "error": "Failed to complete routine deletion"
        }));
    }

    info!("Deleted routine {}", routine_id);
    HttpResponse::Ok().json(json!({ "status": "deleted" }))
}
