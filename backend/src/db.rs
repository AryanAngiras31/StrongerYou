use sqlx::{Error, PgPool};
use std::env;

pub async fn initialize_database() -> Result<PgPool, Error> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("Failed to obtain database url");

    println!("Initializing database connection...");
    let pool = PgPool::connect(&database_url).await?;
    println!("Database connection established.");

    println!("Creating tables if they don't exist...");
    create_tables(&pool).await?;
    println!("Tables initialized.");

    println!("Inserting initial exercise data...");
    insert_initial_exercises(&pool).await?;
    println!("Initial exercise data inserted.");

    Ok(pool)
}

async fn create_tables(pool: &PgPool) -> Result<(), Error> {
    // Users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Users (
            UserID SERIAL PRIMARY KEY,
            DateJoined DATE NOT NULL DEFAULT CURRENT_DATE
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Routines table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Routines(
            RoutineID SERIAL PRIMARY KEY,
            RoutineName VARCHAR(255) NOT NULL,
            Timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UserID INTEGER REFERENCES Users(UserID)
        );
        "#,
    )
    .execute(pool)
    .await?;

    // ExerciseList table with unique constraint on ExerciseName
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
    .execute(pool)
    .await?;

    // Workout table
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
    .execute(pool)
    .await?;

    // PRs table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS PRs (
            PRID SERIAL PRIMARY KEY,
            HeaviestWeight SMALLINT NOT NULL,
            OneRM REAL NOT NULL,
            SetVolume INTEGER NOT NULL,
            ExerciseID INTEGER REFERENCES ExerciseList(ExerciseID),
            WorkoutID INTEGER REFERENCES Workout(WorkoutID)
        );
        "#,
    )
    .execute(pool)
    .await?;

    // HighestRepsPerWeight table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS HighestRepsPerWeight (
            ID SERIAL PRIMARY KEY,
            Weight SMALLINT NOT NULL,
            HighestReps SMALLINT NOT NULL,
            ExerciseID INTEGER REFERENCES ExerciseList(ExerciseID),
            PRID INTEGER REFERENCES PRs(PRID)
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Routines_Exercises_Sets table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Routines_Exercises_Sets (
            RoutineID INTEGER REFERENCES Routines(RoutineID),
            ExerciseID INTEGER REFERENCES ExerciseList(ExerciseID),
            NumberOfSets SMALLINT NOT NULL,
            PRIMARY KEY (RoutineID, ExerciseID)
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Workout_Exercises_Sets table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Workout_Exercises_Sets (
            WorkoutID INTEGER REFERENCES Workout(WorkoutID),
            ExerciseID INTEGER REFERENCES ExerciseList(ExerciseID),
            SetID SMALLINT NOT NULL,
            PRIMARY KEY (WorkoutID, ExerciseID, SetID)
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Set table (Note: "Set" needs to be quoted as it's a reserved keyword)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS "Set" (
            SetID SERIAL PRIMARY KEY,
            Weight SMALLINT NOT NULL,
            Reps SMALLINT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    // MarkerList table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS MarkerList (
            MarkerID SERIAL PRIMARY KEY,
            MarkerName VARCHAR(255) NOT NULL,
            UserID INTEGER REFERENCES Users(UserID),
            Clr VARCHAR(10)
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Markers table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Markers (
            MarkerID INTEGER REFERENCES MarkerList(MarkerID),
            Value REAL NOT NULL,
            Date DATE NOT NULL,
            UserID INTEGER REFERENCES Users(UserID)
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Indices
    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_users_date_joined ON Users(DateJoined);"#)
        .execute(pool)
        .await?;
    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_routines_user ON Routines(UserID);"#)
        .execute(pool)
        .await?;
    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_workout_routine ON Workout(RoutineID);"#)
        .execute(pool)
        .await?;
    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_markers_user ON Markers(UserID);"#)
        .execute(pool)
        .await?;
    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_markers_date ON Markers(Date);"#)
        .execute(pool)
        .await?;

    Ok(())
}

async fn insert_initial_exercises(pool: &PgPool) -> Result<(), Error> {
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
        sqlx::query(
            r#"
            INSERT INTO ExerciseList (ExerciseName, MusclesTrained, ExerciseType)
            VALUES ($1, $2, $3)
            ON CONFLICT (ExerciseName) DO NOTHING
            "#,
        )
        .bind(name)
        .bind(&muscles)
        .bind(type_)
        .execute(pool)
        .await?;
        println!("Inserted exercise: {}", name);
    }

    Ok(())
}
