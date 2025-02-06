use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use serde::{de::value::Error, Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use std::env;
use std::fs;

//mod routines;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8080;
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
            UserID SMALLINT REFERENCES Users(UserID),
            ExerciseList TEXT[] NOT NULL
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
            ExerciseName VARCHAR(255) NOT NULL,
            MusclesTrained TEXT[] NOT NULL
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
            "1RM" REAL NOT NULL,
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
        CREATE TABLE IF NOT EXISTS Markers (
            MarkerID SERIAL PRIMARY KEY,
            MarkerName VARCHAR(255) NOT NULL,
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

    // Start HTTP server
    HttpServer::new(move || {
        App::new().app_data(actix_web::web::Data::new(pool.clone()))
        //.service(routines::create_routine)
        //.service(routines::modify_routine)
        //.service(routines::get_routine_history)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
