use actix_web::{delete, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

// Struct to deserialize incoming JSON for routine creation
#[derive(Deserialize)]
struct CreateRoutineRequest {
    routine_name: String,
    exercise_list: Vec<Exercise>,
}

// Struct representing an exercise in a routine
#[derive(Deserialize, Serialize)]
struct Exercise {
    exercise_name: String,
    exercise_type: String,
    number_of_sets: i32,
}

// Handler for creating a new routine
#[post("/routines/create")]
async fn create_routine(
    pool: web::Data<PgPool>,
    req: web::Json<CreateRoutineRequest>,
) -> impl Responder {
    // Begin a database transaction
    let tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Insert the new routine into the database
    let routine_id = match sqlx::query!(
        "INSERT INTO Routines (RoutineName) VALUES ($1) RETURNING RoutineID",
        req.routine_name
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(record) => record.routineid,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Insert exercises associated with the routine
    for exercise in &req.exercise_list {
        if sqlx::query!(
            "INSERT INTO Routines_Exercises_Sets (RoutineID, ExerciseID, NumberOfSets)
             SELECT $1, ExerciseID, $2 FROM ExerciseList WHERE ExerciseName = $3",
            routine_id as i16,
            exercise.number_of_sets as i16,
            exercise.exercise_name
        )
        .execute(pool.get_ref())
        .await
        .is_err()
        {
            return HttpResponse::InternalServerError().finish();
        }
    }

    HttpResponse::Created().finish()
}

// Handler for modifying a routine by copying an existing one
#[post("/routines/modify")]
async fn modify_routine(
    pool: web::Data<PgPool>,
    req: web::Json<CreateRoutineRequest>,
) -> impl Responder {
    // Get the RoutineID of the existing routine
    let copy_routine_id = match sqlx::query!(
        "SELECT RoutineID FROM Routines WHERE RoutineName = $1",
        req.routine_name
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(record)) => record.routineid,
        _ => return HttpResponse::NotFound().finish(),
    };

    // Insert a new routine with the provided name
    let new_routine_id = match sqlx::query!(
        "INSERT INTO Routines (RoutineName) VALUES ($1) RETURNING RoutineID",
        req.routine_name.clone()
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(record) => record.routineid,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Copy exercises from the old routine to the new one
    sqlx::query!(
        "INSERT INTO Routines_Exercises_Sets (RoutineID, ExerciseID, NumberOfSets)
         SELECT $1, ExerciseID, NumberOfSets FROM Routines_Exercises_Sets WHERE RoutineID = $2",
        new_routine_id as i16,
        copy_routine_id as i16
    )
    .execute(pool.get_ref())
    .await
    .ok();

    HttpResponse::Ok().finish()
}

// Handler for deleting a routine by name
#[delete("/routines/{routine_name}")]
async fn delete_routine(
    pool: web::Data<PgPool>,
    routine_name: web::Path<String>,
) -> impl Responder {
    // Execute the delete query
    let deleted = sqlx::query!(
        "DELETE FROM Routines WHERE RoutineName = $1",
        routine_name.into_inner()
    )
    .execute(pool.get_ref())
    .await;

    // Return appropriate response based on whether rows were affected
    match deleted {
        Ok(res) if res.rows_affected() > 0 => HttpResponse::Ok().finish(),
        _ => HttpResponse::NotFound().finish(),
    }
}
