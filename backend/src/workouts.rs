use actix_web::{get, post, put, web, HttpResponse};
use chrono::{NaiveDateTime, Utc};
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Row};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct Set {
    weight: i16,
    reps: i16,
}

#[derive(Serialize, Deserialize)]
struct Exercise {
    exercise_id: i32,
    exercise_name: String,
    sets: HashMap<i32, Set>,
}

#[derive(Serialize, Deserialize)]
struct WorkoutTemplate {
    exercises: Vec<Exercise>,
}

#[derive(Serialize, Deserialize)]
struct ValidateSetRequest {
    exercise_id: i32,
    weight: i16,
    reps: i16,
}

#[derive(Serialize)]
struct WorkoutSummary {
    workout_id: i32,
    routine_name: String,
    start_time: NaiveDateTime,
}

#[derive(Serialize)]
struct RoutineDetail {
    routine_name: String,
    exercises: Vec<Exercise>,
    routines: Vec<(i32, i16)>, // (exercise_id, number_of_sets)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_workout_template)
        .service(modify_workout)
        .service(finish_workout)
        .service(validate_set)
        .service(display_workouts)
        .service(view_routine);
}

#[get("/workouts/template/{routine_id}")]
async fn get_workout_template(pool: web::Data<PgPool>, routine_id: web::Path<i32>) -> HttpResponse {
    let routine_id = routine_id.into_inner();

    match sqlx::query(
        "SELECT e.ExerciseID, e.ExerciseName, res.NumberOfSets 
         FROM ExerciseList e 
         JOIN Routines_Exercises_Sets res ON e.ExerciseID = res.ExerciseID 
         WHERE res.RoutineID = $1",
    )
    .bind(routine_id)
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => {
            let exercises = rows
                .iter()
                .map(|row| Exercise {
                    exercise_id: row.get("ExerciseID"),
                    exercise_name: row.get("ExerciseName"),
                    sets: (1..=row.get::<i16, _>("NumberOfSets"))
                        .map(|i| (i as i32, Set { weight: 0, reps: 0 }))
                        .collect(),
                })
                .collect();

            HttpResponse::Ok().json(WorkoutTemplate { exercises })
        }
        Err(e) => {
            error!("Failed to fetch workout template: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch workout template"
            }))
        }
    }
}

#[put("/workouts/{workout_id}")]
async fn modify_workout(
    pool: web::Data<PgPool>,
    workout_id: web::Path<i32>,
    workout: web::Json<WorkoutTemplate>,
) -> HttpResponse {
    let workout_id = workout_id.into_inner();
    let mut transaction = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(e) => {
            error!("Failed to start transaction: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Database error"
            }));
        }
    };

    for exercise in &workout.exercises {
        for (set_num, set) in &exercise.sets {
            // Update set
            if let Err(e) = sqlx::query(
                "UPDATE Set s 
                 SET Weight = $1, Reps = $2 
                 FROM Workout_Exercises_Sets wes 
                 WHERE wes.WorkoutID = $3 
                 AND wes.ExerciseID = $4 
                 AND wes.SetID = s.SetID 
                 AND wes.SetID = $5",
            )
            .bind(set.weight)
            .bind(set.reps)
            .bind(workout_id)
            .bind(exercise.exercise_id)
            .bind(*set_num)
            .execute(&mut transaction)
            .await
            {
                error!("Failed to update set: {}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to update workout"
                }));
            }

            // Update PRs and HighestRepsPerWeight
            if let Err(e) = update_prs_and_records(
                &mut transaction,
                exercise.exercise_id,
                workout_id,
                set.weight,
                set.reps,
            )
            .await
            {
                error!("Failed to update PRs: {}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to update records"
                }));
            }
        }
    }

    if let Err(e) = transaction.commit().await {
        error!("Failed to commit transaction: {}", e);
        return HttpResponse::InternalServerError().json(json!({
            "error": "Failed to save changes"
        }));
    }

    HttpResponse::Ok().json(json!({ "status": "updated" }))
}

#[post("/workouts")]
async fn finish_workout(
    pool: web::Data<PgPool>,
    workout: web::Json<WorkoutTemplate>,
) -> HttpResponse {
    let mut transaction = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(e) => {
            error!("Failed to start transaction: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Database error"
            }));
        }
    };

    // Create new workout
    let workout_id = match sqlx::query(
        "INSERT INTO Workout (Start, End, RoutineID) 
         VALUES ($1, $2, $3) 
         RETURNING WorkoutID",
    )
    .bind(Utc::now().naive_utc())
    .bind(Utc::now().naive_utc())
    .bind(1) // Assuming routine_id is provided or defaulted
    .fetch_one(&mut transaction)
    .await
    {
        Ok(row) => row.get::<i32, _>("WorkoutID"),
        Err(e) => {
            error!("Failed to create workout: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to create workout"
            }));
        }
    };

    for exercise in &workout.exercises {
        for (set_num, set) in &exercise.sets {
            // Create set and workout_exercises_sets entries
            if let Err(e) = create_set_and_link(
                &mut transaction,
                workout_id,
                exercise.exercise_id,
                *set_num,
                set.weight,
                set.reps,
            )
            .await
            {
                error!("Failed to create set: {}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to save workout"
                }));
            }

            // Update PRs and records
            if let Err(e) = update_prs_and_records(
                &mut transaction,
                exercise.exercise_id,
                workout_id,
                set.weight,
                set.reps,
            )
            .await
            {
                error!("Failed to update PRs: {}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to update records"
                }));
            }
        }
    }

    if let Err(e) = transaction.commit().await {
        error!("Failed to commit transaction: {}", e);
        return HttpResponse::InternalServerError().json(json!({
            "error": "Failed to save workout"
        }));
    }

    HttpResponse::Created().json(json!({ "workout_id": workout_id }))
}

#[post("/workouts/validate")]
async fn validate_set(pool: web::Data<PgPool>, set: web::Json<ValidateSetRequest>) -> HttpResponse {
    let mut new_prs = HashMap::new();

    // Calculate 1RM using Brzycki formula
    let one_rm = set.weight as f32 * (36.0 / (37.0 - set.reps as f32));
    let set_volume = set.weight as i32 * set.reps as i32;

    // Check PRs
    match sqlx::query(
        "SELECT HeaviestWeight, OneRM, SetVolume 
         FROM PRs 
         WHERE ExerciseID = $1 
         ORDER BY PRID DESC 
         LIMIT 1",
    )
    .bind(set.exercise_id)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(row) => {
            if let Some(row) = row {
                let current_heaviest: i16 = row.get("HeaviestWeight");
                let current_one_rm: f32 = row.get("OneRM");
                let current_volume: i32 = row.get("SetVolume");

                if set.weight > current_heaviest {
                    new_prs.insert("HeaviestWeight", set.weight as f64);
                }
                if one_rm > current_one_rm {
                    new_prs.insert("OneRM", one_rm as f64);
                }
                if set_volume > current_volume {
                    new_prs.insert("SetVolume", set_volume as f64);
                }
            }
        }
        Err(e) => {
            error!("Failed to check PRs: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to validate set"
            }));
        }
    }

    // Check HighestRepsPerWeight
    match sqlx::query(
        "SELECT HighestReps 
         FROM HighestRepsPerWeight 
         WHERE ExerciseID = $1 AND Weight = $2",
    )
    .bind(set.exercise_id)
    .bind(set.weight)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(row) => {
            if let Some(row) = row {
                let current_highest_reps: i16 = row.get("HighestReps");
                if set.reps > current_highest_reps {
                    new_prs.insert(format!("HighestReps@{}kg", set.weight), set.reps as f64);
                }
            } else {
                new_prs.insert(format!("HighestReps@{}kg", set.weight), set.reps as f64);
            }
        }
        Err(e) => {
            error!("Failed to check highest reps: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to validate set"
            }));
        }
    }

    HttpResponse::Ok().json(new_prs)
}

#[get("/workouts")]
async fn display_workouts(pool: web::Data<PgPool>) -> HttpResponse {
    match sqlx::query(
        "SELECT w.WorkoutID, w.Start, r.RoutineName 
         FROM Workout w 
         JOIN Routines r ON w.RoutineID = r.RoutineID 
         ORDER BY w.Start DESC",
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => {
            let workouts: Vec<WorkoutSummary> = rows
                .iter()
                .map(|row| WorkoutSummary {
                    workout_id: row.get("WorkoutID"),
                    routine_name: row.get("RoutineName"),
                    start_time: row.get("Start"),
                })
                .collect();

            HttpResponse::Ok().json(workouts)
        }
        Err(e) => {
            error!("Failed to fetch workouts: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch workouts"
            }))
        }
    }
}

#[get("/routines/{routine_id}")]
async fn view_routine(pool: web::Data<PgPool>, routine_id: web::Path<i32>) -> HttpResponse {
    let routine_id = routine_id.into_inner();

    let routine_name = match sqlx::query("SELECT RoutineName FROM Routines WHERE RoutineID = $1")
        .bind(routine_id)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(row) => row.get::<String, _>("RoutineName"),
        Err(e) => {
            error!("Failed to fetch routine name: {}", e);
            return HttpResponse::NotFound().json(json!({
                "error": "Routine not found"
            }));
        }
    };

    match sqlx::query(
        "SELECT e.ExerciseID, e.ExerciseName, res.NumberOfSets, 
                wes.SetID, s.Weight, s.Reps 
         FROM ExerciseList e 
         JOIN Routines_Exercises_Sets res ON e.ExerciseID = res.ExerciseID 
         LEFT JOIN Workout_Exercises_Sets wes ON e.ExerciseID = wes.ExerciseID 
         LEFT JOIN Set s ON wes.SetID = s.SetID 
         WHERE res.RoutineID = $1",
    )
    .bind(routine_id)
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => {
            let mut exercises = Vec::new();
            let mut routines = Vec::new();

            for row in rows {
                let exercise_id: i32 = row.get("ExerciseID");
                let number_of_sets: i16 = row.get("NumberOfSets");

                exercises.push(Exercise {
                    exercise_id,
                    exercise_name: row.get("ExerciseName"),
                    sets: HashMap::new(), // You might want to populate this based on your needs
                });

                routines.push((exercise_id, number_of_sets));
            }

            HttpResponse::Ok().json(RoutineDetail {
                routine_name,
                exercises,
                routines,
            })
        }
        Err(e) => {
            error!("Failed to fetch routine details: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch routine details"
            }))
        }
    }
}

async fn create_set_and_link(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workout_id: i32,
    exercise_id: i32,
    set_num: i32,
    weight: i16,
    reps: i16,
) -> Result<(), sqlx::Error> {
    // Create new set
    let set_id = sqlx::query("INSERT INTO Set (Weight, Reps) VALUES ($1, $2) RETURNING SetID")
        .bind(weight)
        .bind(reps)
        .fetch_one(transaction)
        .await?
        .get::<i32, _>("SetID");

    // Link set to workout and exercise
    sqlx::query(
        "INSERT INTO Workout_Exercises_Sets (WorkoutID, ExerciseID, SetID) 
         VALUES ($1, $2, $3)",
    )
    .bind(workout_id)
    .bind(exercise_id)
    .bind(set_id)
    .execute(transaction)
    .await?;

    Ok(())
}

async fn update_prs_and_records(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    exercise_id: i32,
    workout_id: i32,
    weight: i16,
    reps: i16,
) -> Result<(), sqlx::Error> {
    // Calculate 1RM using Brzycki formula
    let one_rm = weight as f32 * (36.0 / (37.0 - reps as f32));
    let set_volume = weight as i32 * reps as i32;

    // Get current PRs
    let current_prs = sqlx::query(
        "SELECT HeaviestWeight, OneRM, SetVolume 
         FROM PRs 
         WHERE ExerciseID = $1 
         ORDER BY PRID DESC 
         LIMIT 1",
    )
    .bind(exercise_id)
    .fetch_optional(transaction)
    .await?;

    let should_update_prs = if let Some(row) = current_prs {
        let current_heaviest: i16 = row.get("HeaviestWeight");
        let current_one_rm: f32 = row.get("OneRM");
        let current_volume: i32 = row.get("SetVolume");

        weight > current_heaviest || one_rm > current_one_rm || set_volume > current_volume
    } else {
        true // No existing PRs, so this is automatically a PR
    };

    if should_update_prs {
        // Create new PR record
        let pr_id = sqlx::query(
            "INSERT INTO PRs (HeaviestWeight, OneRM, SetVolume, ExerciseID, WorkoutID) 
             VALUES ($1, $2, $3, $4, $5) 
             RETURNING PRID",
        )
        .bind(weight)
        .bind(one_rm)
        .bind(set_volume)
        .bind(exercise_id)
        .bind(workout_id)
        .fetch_one(transaction)
        .await?
        .get::<i32, _>("PRID");

        // Update HighestRepsPerWeight if necessary
        let current_highest_reps = sqlx::query(
            "SELECT HighestReps 
             FROM HighestRepsPerWeight 
             WHERE ExerciseID = $1 AND Weight = $2",
        )
        .bind(exercise_id)
        .bind(weight)
        .fetch_optional(transaction)
        .await?;

        match current_highest_reps {
            Some(row) => {
                let highest_reps: i16 = row.get("HighestReps");
                if reps > highest_reps {
                    sqlx::query(
                        "UPDATE HighestRepsPerWeight 
                         SET HighestReps = $1, PRID = $2 
                         WHERE ExerciseID = $3 AND Weight = $4",
                    )
                    .bind(reps)
                    .bind(pr_id)
                    .bind(exercise_id)
                    .bind(weight)
                    .execute(transaction)
                    .await?;
                }
            }
            None => {
                sqlx::query(
                    "INSERT INTO HighestRepsPerWeight (Weight, HighestReps, ExerciseID, PRID) 
                     VALUES ($1, $2, $3, $4)",
                )
                .bind(weight)
                .bind(reps)
                .bind(exercise_id)
                .bind(pr_id)
                .execute(transaction)
                .await?;
            }
        }
    }

    Ok(())
}
