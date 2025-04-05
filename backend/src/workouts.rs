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

#[derive(Deserialize)]
struct ValidateSetData {
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
enum PRValue {
    Weight(i16),
    OneRM(f32),
    Volume(i32),
    Reps(i16),
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_workout_template)
        .service(modify_workout)
        .service(finish_workout)
        .service(validate_set)
        .service(display_workouts)
        .service(view_workout);
}

#[post("/workouts/validate")]
async fn validate_set(
    pool: web::Data<PgPool>,
    set_data: web::Json<ValidateSetData>,
) -> HttpResponse {
    let mut new_prs: HashMap<&str, PRValue> = HashMap::new();

    // Calculate 1RM using the formula
    let one_rm = f32::from(set_data.weight) * (36.0 / (37.0 - f32::from(set_data.reps)));
    let set_volume = i32::from(set_data.weight) * i32::from(set_data.reps);

    // Check PRs
    match sqlx::query(
        "SELECT heaviestweight, onerm, setvolume FROM PRs 
         WHERE exerciseid = $1 
         ORDER BY prid DESC LIMIT 1",
    )
    .bind(set_data.exercise_id)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(maybe_pr) => {
            if let Some(pr) = maybe_pr {
                let current_heaviest: i16 = pr.get("heaviestweight");
                let current_one_rm: f32 = pr.get("onerm");
                let current_volume: i32 = pr.get("setvolume");

                if set_data.weight > current_heaviest {
                    new_prs.insert("HeaviestWeight", PRValue::Weight(set_data.weight));
                }
                if one_rm > current_one_rm {
                    new_prs.insert("OneRM", PRValue::OneRM(one_rm));
                }
                if set_volume > current_volume {
                    new_prs.insert("SetVolume", PRValue::Volume(set_volume));
                }
            }
        }
        Err(e) => {
            error!("Database error checking PRs: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to check PRs"
            }));
        }
    }

    // Check HighestRepsPerWeight
    match sqlx::query(
        "SELECT highestreps FROM HighestRepsPerWeight 
         WHERE exerciseid = $1 AND weight = $2",
    )
    .bind(set_data.exercise_id)
    .bind(set_data.weight)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(maybe_record) => {
            if let Some(record) = maybe_record {
                let current_highest_reps: i16 = record.get("highestreps");
                if set_data.reps > current_highest_reps {
                    new_prs.insert("HighestReps", PRValue::Reps(set_data.reps));
                }
            } else {
                new_prs.insert("HighestReps", PRValue::Reps(set_data.reps));
            }
        }
        Err(e) => {
            error!("Database error checking highest reps: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to check highest reps"
            }));
        }
    }

    HttpResponse::Ok().json(new_prs)
}
#[get("/workouts")]
async fn display_workouts(pool: web::Data<PgPool>) -> HttpResponse {
    match sqlx::query(
        r#"SELECT w.workoutid, w.start, r.routinename 
         FROM Workout w 
         LEFT JOIN Routines r ON w.routineid = r.routineid 
         ORDER BY w.start DESC"#,
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => {
            let workouts: Vec<WorkoutSummary> = rows
                .iter()
                .map(|row| WorkoutSummary {
                    workout_id: row.get("workoutid"),
                    routine_name: row.get("routinename"),
                    start_time: row.get("start"),
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

#[get("/workouts/{workout_id}")]
async fn view_workout(pool: web::Data<PgPool>, workout_id: web::Path<i32>) -> HttpResponse {
    let workout_id = workout_id.into_inner();

    let workout_data = sqlx::query(
        r#"SELECT w.routineid, r.routinename, e.exerciseid, e.exercisename, 
           s.weight, s.reps, wes.setid
         FROM Workout w
         JOIN Workout_Exercises_Sets wes ON w.workoutid = wes.workoutid
         JOIN ExerciseList e ON wes.exerciseid = e.exerciseid
         JOIN "Set" s ON wes.setid = s.setid
         LEFT JOIN Routines r ON w.routineid = r.routineid
         WHERE w.workoutid = $1
         ORDER BY e.exerciseid, s.setid"#, // Added ordering to keep sets in order
    )
    .bind(workout_id)
    .fetch_all(pool.get_ref())
    .await;

    match workout_data {
        Ok(rows) => {
            if rows.is_empty() {
                return HttpResponse::NotFound().json(json!({
                    "error": format!("Workout with ID {} not found", workout_id)
                }));
            }

            let mut exercises_map: HashMap<i32, Exercise> = HashMap::new();
            let mut routine_id: Option<i32> = None;
            let mut routine_name: Option<String> = None;

            // Keep track of set count for each exercise
            let mut exercise_set_counter: HashMap<i32, i16> = HashMap::new();

            for row in &rows {
                routine_id = row.try_get("routineid").ok();
                routine_name = row.try_get("routinename").ok();

                let exercise_id: i32 = row.get("exerciseid");
                let exercise = exercises_map.entry(exercise_id).or_insert(Exercise {
                    exercise_id,
                    exercise_name: row.get("exercisename"),
                    sets: HashMap::new(),
                });

                // Increment set number for this exercise
                let set_number = exercise_set_counter.entry(exercise_id).or_insert(0);
                *set_number += 1;

                exercise.sets.insert(
                    *set_number,
                    Set {
                        weight: row.get("weight"),
                        reps: row.get("reps"),
                    },
                );
            }

            HttpResponse::Ok().json(json!({
                "routine_id": routine_id,
                "routine_name": routine_name,
                "exercises": exercises_map.values().collect::<Vec<_>>()
            }))
        }
        Err(e) => {
            error!("Failed to fetch workout details: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to fetch workout details: {}", e)
            }))
        }
    }
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
        for (_set_number, set) in &exercise.sets {
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
