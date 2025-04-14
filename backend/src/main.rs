#![allow(unused_imports)]
use actix_cors::Cors;
use actix_web::cookie::time::error;
use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::{http, middleware};
use dotenv::dotenv;
use env_logger::Env;
use log::{error, info, warn};
use serde::{de::value::Error, Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use std::env;
use std::fs;

mod exercises;
mod markers;
mod routines;
mod workouts;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8081;
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let database_url = env::var("DATABASE_URL").expect("Failed to obtain database url");
    //print!("database url : {}\n", database_url);

    //Create database pool
    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to form database pool");

    //Initialize tables
    //sqlx::query(
    //    r#"
    //    CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
    //    "#,
    //)
    //.execute(&pool)
    //.await
    //.expect("Failed to create extension");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Users (
            UserID SERIAL PRIMARY KEY,
            DateJoined DATE NOT NULL DEFAULT CURRENT_DATE
        );    
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create table Routines");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Routines(
            RoutineID SERIAL PRIMARY KEY,
            RoutineName VARCHAR(255) NOT NULL,
            Timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UserID SMALLINT REFERENCES Users(UserID)
        );      
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create table Routines");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ExerciseList (
            ExerciseID SERIAL PRIMARY KEY,
            ExerciseName VARCHAR(255) UNIQUE NOT NULL,
            MusclesTrained TEXT[] NOT NULL,
            ExerciseType VARCHAR(255) NOT NULL
        );
    
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create table ExerciseList");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Workout (
            WorkoutID SERIAL PRIMARY KEY,
            Start TIMESTAMP NOT NULL,
            "End" TIMESTAMP NOT NULL,
            RoutineID INTEGER REFERENCES Routines(RoutineID)
        );    
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create table ExerciseList");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS PRs (
            PRID SERIAL PRIMARY KEY,
            HeaviestWeight SMALLINT NOT NULL,
            OneRM REAL NOT NULL,
            SetVolume INTEGER NOT NULL,
            ExerciseID SMALLINT REFERENCES ExerciseList(ExerciseID),
            WorkoutID SMALLINT REFERENCES Workout(WorkoutID)
        );    
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create table ExerciseList");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS HighestRepsPerWeight (
            ID SERIAL PRIMARY KEY,
            Weight SMALLINT NOT NULL,
            HighestReps SMALLINT NOT NULL,
            ExerciseID SMALLINT REFERENCES ExerciseList(ExerciseID),
            PRID SMALLINT REFERENCES PRs(PRID)
        );  
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create table ExerciseList");

    sqlx::query(
        r#" 
        CREATE TABLE IF NOT EXISTS Routines_Exercises_Sets (
            RoutineID SMALLINT REFERENCES Routines(RoutineID),
            ExerciseID SMALLINT REFERENCES ExerciseList(ExerciseID),
            NumberOfSets SMALLINT NOT NULL,
            PRIMARY KEY (RoutineID, ExerciseID)
        );    
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create table ExerciseList");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Workout_Exercises_Sets (
            WorkoutID SMALLINT REFERENCES Workout(WorkoutID),
            ExerciseID SMALLINT REFERENCES ExerciseList(ExerciseID),
            SetID SMALLINT,
            PRIMARY KEY (WorkoutID, ExerciseID, SetID)
        );    
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create table ExerciseList");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS "Set" (
            SetID SERIAL PRIMARY KEY,
            Weight SMALLINT NOT NULL,
            Reps SMALLINT NOT NULL
        );    
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create table ExerciseList");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS MarkerList (
            MarkerID SERIAL PRIMARY KEY,
            MarkerName VARCHAR(255) NOT NULL,
            UserID SMALLINT REFERENCES Users(UserID),
            Clr VARCHAR(10) 
        );     
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create table ExerciseList");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Markers (
            MarkerID INTEGER REFERENCES MarkerList(MarkerID),
            Value REAL NOT NULL,
            Date DATE NOT NULL,
            UserID SMALLINT REFERENCES Users(UserID)
        );     
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create table ExerciseList");

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_users_date_joined ON Users(DateJoined);
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create indexes");

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_routines_user ON Routines(UserID);
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create indexes");

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_workout_routine ON Workout(RoutineID);
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create indexes");

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_markers_user ON Markers(UserID);
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create indexes");

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_markers_date ON Markers(Date);
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create indexes");

    let exercises = vec![
        (
            "Bench Press",
            vec!["Chest", "Shoulders", "Triceps"],
            "Regular",
        ),
        (
            "Incline Press (Dumbbell)",
            vec!["Chest", "Shoulders", "Triceps"],
            "Single limb",
        ),
        (
            "Incline Press (Smith Machine)",
            vec!["Chest", "Shoulders", "Triceps"],
            "Regular",
        ),
        (
            "Flat Press (Dumbbell)",
            vec!["Chest", "Shoulders", "Triceps"],
            "Single limb",
        ),
        (
            "Flat Press (Smith Machine)",
            vec!["Chest", "Shoulders", "Triceps"],
            "Regular",
        ),
        (
            "Seated Dips",
            vec!["Chest", "Triceps", "Shoulders"],
            "Regular",
        ),
        ("Standing Cable Chest Fly", vec!["Chest"], "Regular"),
        (
            "Barbell Squat",
            vec!["Quads", "Glutes", "Hamstrings"],
            "Regular",
        ),
        (
            "Romanian Deadlift",
            vec!["Hamstrings", "Glutes", "Lower Back"],
            "Regular",
        ),
        (
            "Leg Press",
            vec!["Quads", "Glutes", "Hamstrings"],
            "Regular",
        ),
        ("Calf Raises", vec!["Calves"], "Regular"),
        ("Pull-ups", vec!["Back", "Biceps"], "Regular"),
        ("Barbell Rows", vec!["Back", "Biceps"], "Regular"),
        ("Lat Pulldown", vec!["Back", "Biceps"], "Regular"),
        ("Bicep Curls (Dumbbell)", vec!["Biceps"], "Single limb"),
        (
            "Hammer Curls (Dumbbell)",
            vec!["Biceps", "Forearms"],
            "Single limb",
        ),
        ("Tricep Extensions (Cable)", vec!["Triceps"], "Regular"),
        (
            "Overhead Press (Barbell)",
            vec!["Shoulders", "Triceps"],
            "Regular",
        ),
        (
            "Lateral Raises (Dumbbell)",
            vec!["Shoulders"],
            "Single limb",
        ),
        ("Front Raises (Dumbbell)", vec!["Shoulders"], "Single limb"),
    ];

    for (name, muscles, type_) in exercises {
        let result = sqlx::query(
            r#"
            INSERT INTO ExerciseList (ExerciseName, MusclesTrained, ExerciseType)
            VALUES ($1, $2, $3)
            ON CONFLICT (ExerciseName) DO NOTHING
            "#,
        )
        .bind(name)
        .bind(&muscles)
        .bind(type_)
        .execute(&pool)
        .await;

        match result {
            Ok(_) => {
                info!("Successfully inserted exercise: {}", name);
            }
            Err(e) => {
                error!("Failed to insert exercise {}: {}", name, e);
            }
        }
    }

    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:8101") // Your frontend's origin
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
            .max_age(3600);
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .wrap(Logger::default()) // Enable logging
            .app_data(web::Data::new(pool.clone()))
            .configure(configure_routes)
    })
    .bind(("localhost", port))?
    .run()
    .await
}

fn configure_routes(cfg: &mut web::ServiceConfig) {
    exercises::init_routes(cfg);
    markers::init_routes(cfg);
    routines::init_routes(cfg);
    workouts::init_routes(cfg);
}
