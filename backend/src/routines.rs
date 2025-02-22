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
    exercises: Vec<RoutineExercise>,
}

#[derive(Serialize, Deserialize)]
struct RoutineUpdate {
    name: String,
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
    timestamp: NaiveDateTime,
    last_performed: Option<NaiveDate>,
}

#[derive(Serialize)]
struct RoutineDetail {
    routine_id: i32,
    name: String,
    timestamp: NaiveDateTime,
    exercises: Vec<RoutineExerciseDetail>,
}

#[derive(Serialize, FromRow)]
struct RoutineExerciseDetail {
    exercise_id: i32,
    exercise_name: String,
    sets: i32,
}

#[derive(Serialize, FromRow)]
struct SetDetail {
    weight: i32,
    reps: i32,
}

#[derive(Serialize, FromRow)]
struct ExerciseDetail {
    exercise_id: i32,
    exercise_name: String,
    sets: Vec<SetDetail>,
}

#[derive(Serialize)]
struct RoutineViewResponse {
    routine_id: i32,
    name: String,
    timestamp: NaiveDateTime,
    exercises: Vec<ExerciseDetail>,
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
    // Check if we need to include lastPerformed
    let include_last_performed = request.get("include") == Some(&"lastPerformed".to_string());

    // Query to get routines with optional last performed date
    let query = if include_last_performed {
        "SELECT r.RoutineID, r.RoutineName, r.Timestamp, 
         (SELECT MAX(w.Start::date) FROM Workout w WHERE w.RoutineID = r.RoutineID) as last_performed
         FROM Routines r ORDER BY r.Timestamp ASC"
    } else {
        "SELECT r.RoutineID, r.RoutineName, r.Timestamp,
         NULL as last_performed
         FROM Routines r ORDER BY r.Timestamp ASC"
    };

    match sqlx::query(query).fetch_all(pool.get_ref()).await {
        Ok(rows) => {
            let routines: Vec<RoutineInfo> = rows
                .iter()
                .map(|row| RoutineInfo {
                    routine_id: row.get("routineid"),
                    name: row.get("routinename"),
                    timestamp: row.get("timestamp"),
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

#[get("/routines/name")]
async fn get_routine_by_name(
    pool: web::Data<PgPool>,
    request: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    // Get routine by name
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
        "INSERT INTO Routines (RoutineName, Timestamp) 
         VALUES ($1, $2) RETURNING RoutineID",
    )
    .bind(&routine.name)
    .bind(Utc::now().naive_utc())
    .fetch_one(&mut *tx)
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
            "INSERT INTO Routines_Exercises_Sets (RoutineID, ExerciseID, NumberOfSets) 
             VALUES ($1, $2, $3)",
        )
        .bind(routine_id)
        .bind(exercise.exercise_id)
        .bind(exercise.sets)
        .execute(&mut *tx)
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
    if let Err(e) = sqlx::query("UPDATE Routines SET RoutineName = $1 WHERE RoutineID = $2")
        .bind(&update.name)
        .bind(routine_id)
        .execute(&mut *tx)
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
        .execute(&mut *tx)
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
            "INSERT INTO Routines_Exercises_Sets (RoutineID, ExerciseID, NumberOfSets) 
             VALUES ($1, $2, $3)",
        )
        .bind(routine_id)
        .bind(exercise.exercise_id)
        .bind(exercise.sets)
        .execute(&mut *tx)
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
        .execute(&mut *tx)
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
        .execute(&mut *tx)
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

#[get("/routines/{routine_id}")]
async fn view_routine(pool: web::Data<PgPool>, routine_id: web::Path<i32>) -> HttpResponse {
    let routine_id = routine_id.into_inner();

    // Fetch routine details
    let routine = match sqlx::query!(
        r#"SELECT RoutineID, RoutineName, Timestamp FROM Routines WHERE RoutineID = $1"#,
        routine_id
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(row) => row,
        Err(e) => {
            error!("Failed to fetch routine details: {}", e);
            return HttpResponse::NotFound().json(json!({
                "error": format!("Routine with ID {} not found", routine_id)
            }));
        }
    };

    // Fetch exercises and their sets for the routine
    let exercises = match sqlx::query!(
        r#"
        SELECT e.ExerciseID, e.ExerciseName, s.SetID, s.Weight, s.Reps
        FROM Routines_Exercises_Sets res
        JOIN ExerciseList e ON res.ExerciseID = e.ExerciseID
        LEFT JOIN "Set" s ON res.RoutineID = s.SetID
        WHERE res.RoutineID = $1
        ORDER BY e.ExerciseID, s.SetID
        "#,
        routine_id
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => {
            let mut exercises_map: HashMap<i32, ExerciseDetail> = HashMap::new();

            for row in rows {
                let exercise_id = row.exercise_id;
                let exercise_name = row.exercise_name;
                let set = SetDetail {
                    weight: row.weight.unwrap_or(0), // Default to 0 if weight is NULL
                    reps: row.reps.unwrap_or(0),     // Default to 0 if reps is NULL
                };

                exercises_map
                    .entry(exercise_id)
                    .or_insert(ExerciseDetail {
                        exercise_id,
                        exercise_name,
                        sets: Vec::new(),
                    })
                    .sets
                    .push(set);
            }

            exercises_map.into_values().collect::<Vec<ExerciseDetail>>()
        }
        Err(e) => {
            error!("Failed to fetch exercises and sets: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch routine exercises and sets"
            }));
        }
    };

    // Construct the response
    let response = RoutineViewResponse {
        routine_id: routine.routine_id,
        name: routine.routine_name,
        timestamp: routine.timestamp,
        exercises,
    };

    info!("Retrieved details for routine {}", routine_id);
    HttpResponse::Ok().json(response)
}
