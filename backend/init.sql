-- Create the main application database
CREATE DATABASE mobile_app_db;

-- Create application user
CREATE USER app_user WITH PASSWORD 'secure_development_password';

-- Grant privileges to the application user
GRANT ALL PRIVILEGES ON DATABASE mobile_app_db TO app_user;

-- Optional: Create initial tables
-- Fitness Tracker Database Initialization Script

-- Create extensions for array support and UUID generation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users Table
CREATE TABLE Users (
    UserID SERIAL PRIMARY KEY,
    DateJoined DATE NOT NULL DEFAULT CURRENT_DATE
);

-- Routines Table
CREATE TABLE Routines (
    RoutineID SERIAL PRIMARY KEY,
    RoutineName VARCHAR(255) NOT NULL,
    Timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UserID SMALLINT REFERENCES Users(UserID),
    ExerciseList TEXT[] NOT NULL
);

-- Exercise List Table
CREATE TABLE ExerciseList (
    ExerciseID SERIAL PRIMARY KEY,
    ExerciseName VARCHAR(255) NOT NULL,
    MusclesTrained TEXT[] NOT NULL
);

-- Workout Table
CREATE TABLE Workout (
    WorkoutID SERIAL PRIMARY KEY,
    Start TIMESTAMP NOT NULL,
    End TIMESTAMP NOT NULL,
    RoutineID SMALLINT REFERENCES Routines(RoutineID)
);

-- Personal Records (PRs) Table
CREATE TABLE PRs (
    PRID SERIAL PRIMARY KEY,
    HeaviestWeight SMALLINT NOT NULL,
    "1RM" REAL NOT NULL,
    SetVolume INTEGER NOT NULL,
    ExerciseID SMALLINT REFERENCES ExerciseList(ExerciseID),
    WorkoutID SMALLINT REFERENCES Workout(WorkoutID)
);

-- Highest Reps Per Weight Table
CREATE TABLE HighestRepsPerWeight (
    ID SERIAL PRIMARY KEY,
    Weight SMALLINT NOT NULL,
    HighestReps SMALLINT NOT NULL,
    ExerciseID SMALLINT REFERENCES ExerciseList(ExerciseID),
    PRID SMALLINT REFERENCES PRs(PRID)
);

-- Routines Exercises Sets Table
CREATE TABLE Routines_Exercises_Sets (
    RoutineID SMALLINT REFERENCES Routines(RoutineID),
    ExerciseID SMALLINT REFERENCES ExerciseList(ExerciseID),
    NumberOfSets SMALLINT NOT NULL,
    PRIMARY KEY (RoutineID, ExerciseID)
);

-- Workout Exercises Sets Table
CREATE TABLE Workout_Exercises_Sets (
    WorkoutID SMALLINT REFERENCES Workout(WorkoutID),
    ExerciseID SMALLINT REFERENCES ExerciseList(ExerciseID),
    SetID SMALLINT,
    PRIMARY KEY (WorkoutID, ExerciseID, SetID)
);

-- Set Table
CREATE TABLE "Set" (
    SetID SERIAL PRIMARY KEY,
    Weight SMALLINT NOT NULL,
    Reps SMALLINT NOT NULL
);

-- Markers Table
CREATE TABLE Markers (
    MarkerID SERIAL PRIMARY KEY,
    MarkerName VARCHAR(255) NOT NULL,
    Value REAL NOT NULL,
    Date DATE NOT NULL,
    UserID SMALLINT REFERENCES Users(UserID)
);

-- Add indexes to improve query performance
CREATE INDEX idx_users_date_joined ON Users(DateJoined);
CREATE INDEX idx_routines_user ON Routines(UserID);
CREATE INDEX idx_workout_routine ON Workout(RoutineID);
CREATE INDEX idx_markers_user ON Markers(UserID);
CREATE INDEX idx_markers_date ON Markers(Date);

-- Optional: Add some sample data for testing
INSERT INTO Users (DateJoined) VALUES 
    (CURRENT_DATE),
    (CURRENT_DATE - INTERVAL '1 month');

INSERT INTO ExerciseList (ExerciseName, MusclesTrained) VALUES 
    ('Bench Press', ARRAY['Chest', 'Triceps', 'Shoulders']),
    ('Squats', ARRAY['Quadriceps', 'Hamstrings', 'Glutes']),
    ('Deadlift', ARRAY['Back', 'Hamstrings', 'Glutes']);

-- Add more sample data as needed for other tables