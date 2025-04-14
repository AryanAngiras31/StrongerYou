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
    timestamp: string; // Assuming NaiveDateTime serializes to a string
    last_performed: string | null; // Assuming Option<NaiveDate> serializes to string or null
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
    start_time?: string | null; // Assuming Option<NaiveDateTime> serializes to string or null
    end_time?: string | null;   // Assuming Option<NaiveDateTime> serializes to string or null
    routine_id?: number | null; // Assuming Option<i32> serializes to number or null
  }
  
  export interface ValidateSetData {
    exercise_id: number;
    weight: number; // Assuming i16 becomes number in JS
    reps: number;   // Assuming i16 becomes number in JS
  }
  
  export type PRValue =
    | { type: 'Weight'; value: number }   // Assuming i16 becomes number in JS
    | { type: 'OneRM'; value: number }
    | { type: 'Volume'; value: number }
    | { type: 'Reps'; value: number };    // Assuming i16 becomes number in JS
  
  export interface WorkoutSummary {
    workout_id: number;
    routine_name: string;
    start_time: string; // Assuming NaiveDateTime serializes to a string
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
    value: number; // Assuming f64 becomes number in JS
    date: string;   // Assuming NaiveDate serializes to a string
  }
  
  export interface TimelineEntry {
    value: number; // Assuming f64 becomes number in JS
    date: string;
  }
  
  export type MetricType = 'average' | 'sum';
  
  export interface MarkerAnalyticsResponse {
    sum?: number;
    average?: number;
  }