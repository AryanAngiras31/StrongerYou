use actix_web::{get, post, put, web, HttpResponse};
use chrono::NaiveDateTime;
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
    sets: HashMap<i16, Set>, // set_number -> (weight, reps)
}

#[derive(Serialize, Deserialize)]
struct WorkoutData {
    exercises: Vec<Exercise>,
    start_time: Option<NaiveDateTime>,
    end_time: Option<NaiveDateTime>,
    routine_id: Option<i32>,
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_workout_template)
        .service(modify_workout)
        .service(finish_workout);
}

#[get("/workouts/template/{routine_id}")]
async fn get_workout_template(pool: web::Data<PgPool>, routine_id: web::Path<i32>) -> HttpResponse {
    let routine_id = routine_id.into_inner();

    // First verify the routine exists
    match sqlx::query("SELECT routineid FROM Routines WHERE routineid = $1")
        .bind(routine_id)
        .fetch_optional(pool.get_ref())
        .await
    {
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "error": format!("Routine with ID {} not found", routine_id)
            }));
        }
        Err(e) => {
            error!("Database error: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Internal server error"
            }));
        }
        Ok(Some(_)) => {}
    }

    match sqlx::query(
        r#"SELECT e.exerciseid, e.exercisename, r.numberofsets 
         FROM ExerciseList e
         JOIN Routines_Exercises_Sets r ON e.exerciseid = r.exerciseid
         WHERE r.routineid = $1"#,
    )
    .bind(routine_id)
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => {
            let exercises: Vec<Exercise> = rows
                .iter()
                .map(|row| {
                    let number_of_sets: i16 = row.get("numberofsets");
                    let mut sets = HashMap::new();

                    for set_num in 1..=number_of_sets {
                        sets.insert(set_num, Set { weight: 0, reps: 0 });
                    }

                    Exercise {
                        exercise_id: row.get("exerciseid"),
                        exercise_name: row.get("exercisename"),
                        sets,
                    }
                })
                .collect();

            HttpResponse::Ok().json(json!({
                "exercises": exercises
            }))
        }
        Err(e) => {
            error!("Failed to fetch workout template: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch workout template"
            }))
        }
    }
}

async fn update_prs(
    pool: &PgPool,
    workout_id: i32,
    exercise: &Exercise,
) -> Result<(), sqlx::Error> {
    let mut heaviest_weight = 0i16;
    let mut highest_volume = 0i32;
    let mut highest_reps_map: HashMap<i16, i16> = HashMap::new();

    for (_, set) in &exercise.sets {
        if set.weight > heaviest_weight {
            heaviest_weight = set.weight;
        }

        highest_volume += i32::from(set.weight) * i32::from(set.reps);

        let current_highest = highest_reps_map.entry(set.weight).or_insert(0);
        if set.reps > *current_highest {
            *current_highest = set.reps;
        }
    }

    let one_rm = exercise
        .sets
        .values()
        .map(|set| {
            let weight = f32::from(set.weight);
            let reps = f32::from(set.reps);
            weight * (36.0 / (37.0 - reps))
        })
        .fold(0.0, f32::max);

    let pr_id: i32 = sqlx::query(
        "INSERT INTO PRs (heaviestweight, onerm, setvolume, exerciseid, workoutid)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING prid",
    )
    .bind(heaviest_weight)
    .bind(one_rm)
    .bind(highest_volume)
    .bind(exercise.exercise_id)
    .bind(workout_id)
    .fetch_one(pool)
    .await?
    .get("prid");

    for (weight, reps) in highest_reps_map {
        sqlx::query(
            "INSERT INTO HighestRepsPerWeight (weight, highestreps, exerciseid, prid)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (exerciseid, weight)
             DO UPDATE SET highestreps = EXCLUDED.highestreps, prid = EXCLUDED.prid
             WHERE HighestRepsPerWeight.highestreps < EXCLUDED.highestreps",
        )
        .bind(weight)
        .bind(reps)
        .bind(exercise.exercise_id)
        .bind(pr_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn validate_routine_id(pool: &PgPool, routine_id: i32) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("SELECT routineid FROM Routines WHERE routineid = $1")
        .bind(routine_id)
        .fetch_optional(pool)
        .await?;
    Ok(result.is_some())
}

async fn save_workout_data(
    pool: &PgPool,
    workout_data: &WorkoutData,
    workout_id: Option<i32>,
) -> Result<i32, sqlx::Error> {
    // Validate routine_id if provided
    if let Some(routine_id) = workout_data.routine_id {
        if !validate_routine_id(pool, routine_id).await? {
            return Err(sqlx::Error::Protocol(format!(
                "Routine with ID {} does not exist",
                routine_id
            )));
        }
    }

    let workout_id = match workout_id {
        Some(id) => id,
        None => sqlx::query(
            r#"INSERT INTO Workout (start, "end", routineid)
                 VALUES ($1, $2, $3)
                 RETURNING workoutid"#,
        )
        .bind(workout_data.start_time)
        .bind(workout_data.end_time)
        .bind(workout_data.routine_id)
        .fetch_one(pool)
        .await?
        .get("workoutid"),
    };

    for exercise in &workout_data.exercises {
        for (set_number, set) in &exercise.sets {
            let set_id: i32 = sqlx::query(
                r#"INSERT INTO "Set" (weight, reps)
                 VALUES ($1, $2)
                 RETURNING setid"#,
            )
            .bind(set.weight)
            .bind(set.reps)
            .fetch_one(pool)
            .await?
            .get("setid");

            sqlx::query(
                "INSERT INTO Workout_Exercises_Sets (workoutid, exerciseid, setid)
                 VALUES ($1, $2, $3)",
            )
            .bind(workout_id)
            .bind(exercise.exercise_id)
            .bind(set_id)
            .execute(pool)
            .await?;
        }

        update_prs(pool, workout_id, exercise).await?;
    }

    Ok(workout_id)
}

#[put("/workouts/{workout_id}")]
async fn modify_workout(
    pool: web::Data<PgPool>,
    workout_id: web::Path<i32>,
    workout_data: web::Json<WorkoutData>,
) -> HttpResponse {
    let workout_id = workout_id.into_inner();

    match save_workout_data(pool.get_ref(), &workout_data, Some(workout_id)).await {
        Ok(_) => {
            info!("Updated workout {}", workout_id);
            HttpResponse::Ok().json(json!({ "status": "updated" }))
        }
        Err(e) => {
            error!("Failed to update workout: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to update workout: {}", e)
            }))
        }
    }
}

#[post("/workouts")]
async fn finish_workout(
    pool: web::Data<PgPool>,
    workout_data: web::Json<WorkoutData>,
) -> HttpResponse {
    match save_workout_data(pool.get_ref(), &workout_data, None).await {
        Ok(workout_id) => {
            info!("Created new workout {}", workout_id);
            HttpResponse::Created().json(json!({ "workout_id": workout_id }))
        }
        Err(e) => {
            error!("Failed to create workout: {}", e);
            if e.to_string().contains("does not exist") {
                HttpResponse::BadRequest().json(json!({
                    "error": e.to_string()
                }))
            } else {
                HttpResponse::InternalServerError().json(json!({
                    "error": format!("Failed to create workout: {}", e)
                }))
            }
        }
    }
}
