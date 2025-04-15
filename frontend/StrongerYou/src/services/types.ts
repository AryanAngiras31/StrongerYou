// types.ts

// Routines Module
export interface RoutineCreate {
  name: string;
  exercises: RoutineExercise[];
}

export interface RoutineUpdate {
  name: string;
  exercises: RoutineExercise[];
}

export interface RoutineExercise {
  exercise_id: number;
  sets: number;
}

export interface RoutineInfo {
  routine_id: number;
  name: string;
  timestamp: string;
  last_performed: string | null;
}

export interface RoutineExerciseDetail {
  exercise_id: number;
  exercise_name: string;
  sets: number;
}

export interface RoutineViewResponse {
  routine_id: number;
  routine_name: string;
  routines: ExerciseSetPair[];
}

export interface ExerciseSetPair {
  exercise_id: number;
  num_sets: number;
}

// Workouts Module
export interface Set {
  weight: number;
  reps: number;
}

export interface Exercise {
  exercise_id: number;
  exercise_name: string;
  sets: Record<number, Set>; // HashMap<i16, Set> becomes Record<number, Set>
}

export interface WorkoutData {
  exercises: Exercise[];
  start_time?: string | null;
  end_time?: string | null;
  routine_id?: number | null;
}

export interface WorkoutTemplate {
  exercises: Exercise[];
}

export interface ValidateSetData {
  exercise_id: number;
  weight: number;
  reps: number;
}

export type PRValue =
  | { type: 'Weight'; value: number }
  | { type: 'OneRM'; value: number }
  | { type: 'Volume'; value: number }
  | { type: 'Reps'; value: number };

export interface WorkoutSummary {
  workout_id: number;
  routine_name: string;
  start_time: string;
}

// Markers Module
export interface MarkerCreate {
  name: string;
  color: string; // Hex color
}

export interface MarkerUpdate {
  name: string;
  color: string;
}

export interface MarkerValue {
  value: number;
  date: string;
}

export interface TimelineEntry {
  value: number;
  date: string;
}

export type MetricType = 'average' | 'sum';

export interface MarkerAnalyticsResponse {
  sum?: number;
  average?: number;
}

// Exercises Module Types (New additions from exercises.rs)
export interface ExerciseInput {
  exercise_name: string;
  muscles_trained: string[];
  exercise_type: string;
}

export interface ExerciseStats {
  date: string; // Assuming DateTime<Utc> serializes to a string
  value: number; // Changed from f64 to number for consistency
}

export interface PersonalRecord {
  workout_date: string; // Assuming DateTime<Utc> serializes to a string
  weight: number; // Changed from i16 to number for consistency
  reps: number;   // Changed from i16 to number for consistency
  one_rm: number; // Changed from f32 to number for consistency
  set_volume: number; // Changed from i32 to number for consistency
}

export interface ExerciseSearchResult {
  exerciseid: number;
  exercisename: string;
  muscles_trained: string[];
}

export interface ExerciseIdResult {
  exerciseid: number;
}

export interface ExerciseDetails {
  exerciseid: number;
  exercisename: string;
  muscles_trained: string[];
  exercisetype: string;
}

export interface DeletedExercise {
  exerciseid: number;
}