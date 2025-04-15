import axios from 'axios';
import * as type from '../services/types';
const BASE_URL = 'https://localhost:8080';

// Routines API

// 1. Fetch RoutineID by RoutineName
export const get_routine_by_name = async (name: string): Promise<{ routine_id: number }> => {
  try {
    const response = await axios.get<{ routine_id: number }>(`${BASE_URL}/routines/name`, {
      params: { name },
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      },
    });
    return response.data;
  } catch (error) {
    console.error('Error fetching routine by name:', error);
    throw error;
  }
};

// 2. Create Routine
export const create_routine = async (routineData: type.RoutineCreate): Promise<{ routine_id: number }> => {
  try {
    const response = await axios.post<{ routine_id: number }>(`${BASE_URL}/routines`, routineData, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error creating routine:', error);
    throw error;
  }
};

// 3. Modify Routine
export const update_routine = async (routineId: number, routineData: type.RoutineUpdate): Promise<{ status: string }> => {
  try {
    const response = await axios.put<{ status: string }>(`${BASE_URL}/routines/${routineId}`, routineData, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error updating routine:', error);
    throw error;
  }
};

// 4. Delete Routine
export const delete_routine = async (routineId: number): Promise<{ status: string }> => {
  try {
    const response = await axios.delete<{ status: string }>(`${BASE_URL}/routines/${routineId}`, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error deleting routine:', error);
    throw error;
  }
};

// 5. Display Routines
export const list_routines = async (includeLastPerformed: boolean = false): Promise<type.RoutineInfo[]> => {
  try {
    const response = await axios.get<type.RoutineInfo[]>(`${BASE_URL}/routines`, {
      params: {
        include: includeLastPerformed ? 'lastPerformed' : undefined,
      },
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error listing routines:', error);
    throw error;
  }
};

// 6. View Routine
export const view_routine = async (routineId: number): Promise<type.RoutineViewResponse> => {
  try {
    const response = await axios.get<type.RoutineViewResponse>(`${BASE_URL}/routines/${routineId}`, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error viewing routine:', error);
    throw error;
  }
};

// Workouts API

// 1. Get workout template
export const start_a_new_workout = async (routineId: number): Promise<type.WorkoutTemplate> => {
    try {
      const response = await axios.get<type.WorkoutTemplate>(`<span class="math-inline">\{BASE\_URL\}/workouts/template/</span>{routineId}`, {
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json'
        }
      });
      return response.data;
    } catch (error) {
      console.error('Error starting a new workout:', error);
      throw error;
    }
  };

// 2. Modify an Existing Workout
export const modify_workout = async (workoutId: number, workoutData: type.WorkoutData): Promise<{ status: string }> => {
  try {
    const response = await axios.put<{ status: string }>(`${BASE_URL}/workouts/${workoutId}`, workoutData, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error modifying workout:', error);
    throw error;
  }
};

// 3. Finish Workout
export const finish_workout = async (workoutData: type.WorkoutData): Promise<{ workout_id: number }> => {
  try {
    const response = await axios.post<{ workout_id: number }>(`${BASE_URL}/workouts`, workoutData, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error finishing workout:', error);
    throw error;
  }
};

// 4. Validate Set
export const validate_set = async (setData: type.ValidateSetData): Promise<Record<string, type.PRValue>> => {
  try {
    const response = await axios.post<Record<string, type.PRValue>>(`${BASE_URL}/workouts/validate`, setData, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error validating set:', error);
    throw error;
  }
};

// 5. Display Workouts
export const display_workouts = async (): Promise<type.WorkoutSummary[]> => {
  try {
    const response = await axios.get<type.WorkoutSummary[]>(`${BASE_URL}/workouts`, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error displaying workouts:', error);
    throw error;
  }
};

// 6. View Workout
export const view_workout = async (workoutId: number): Promise<{ routine_id: number | null; routine_name: string | null; exercises: type.Exercise[] }> => {
  try {
    const response = await axios.get<{ routine_id: number | null; routine_name: string | null; exercises: type.Exercise[] }>(`${BASE_URL}/workouts/${workoutId}`, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error viewing workout:', error);
    throw error;
  }
};

// Exercises API

// 1. Search for exercises by partial name
export const search_exercises_by_name = async (partial_name: string): Promise<type.ExerciseSearchResult[]> => {
  try {
    const response = await axios.get<type.ExerciseSearchResult[]>(`${BASE_URL}/exercises/search/${partial_name}`, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      },
    });
    return response.data;
  } catch (error) {
    console.error('Error searching exercises by name:', error);
    throw error;
  }
};

// 2. Get exercise ID by exact name
export const get_exercise_id_by_name = async (exercise_name: string): Promise<{ exerciseid: number } | null> => {
  try {
    const response = await axios.get<{ exerciseid: number }>(`${BASE_URL}/exercises/id/${exercise_name}`, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      },
    });
    return response.data;
  } catch (error) {
    console.error('Error fetching exercise ID by name:', error);
    return null;
  }
};

// 3. Create a new exercise
export const create_exercise = async (exerciseData: type.ExerciseInput): Promise<type.ExerciseDetails> => {
  try {
    const response = await axios.post<type.ExerciseDetails>(`${BASE_URL}/exercises`, exerciseData, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error creating exercise:', error);
    throw error;
  }
};

// 4. Delete an exercise by ID
export const delete_exercise = async (exercise_id: number): Promise<type.DeletedExercise> => {
  try {
    const response = await axios.delete<type.DeletedExercise>(`${BASE_URL}/exercises/${exercise_id}`, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error deleting exercise:', error);
    throw error;
  }
};

// 5. Get set volume history for an exercise by ID
export const get_exercise_volume = async (exercise_id: number): Promise<type.ExerciseStats[]> => {
  try {
    const response = await axios.get<type.ExerciseStats[]>(`${BASE_URL}/exercises/volume/${exercise_id}`, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error fetching exercise volume:', error);
    throw error;
  }
};

// 6. Get max weight history for an exercise by ID
export const get_exercise_max_weight = async (exercise_id: number): Promise<type.ExerciseStats[]> => {
  try {
    const response = await axios.get<type.ExerciseStats[]>(`${BASE_URL}/exercises/max-weight/${exercise_id}`, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error fetching exercise max weight:', error);
    throw error;
  }
};

// 7. Get PRs for an exercise by ID
export const get_exercise_prs = async (exercise_id: number): Promise<type.PersonalRecord[]> => {
  try {
    const response = await axios.get<type.PersonalRecord[]>(`${BASE_URL}/exercises/prs/${exercise_id}`, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error fetching exercise PRs:', error);
    throw error;
  }
};

// Markers API

// 1. Get marker ID by name
export const get_marker_by_name = async (name: string): Promise<{ marker_id: number } | null> => {
  try {
    const response = await axios.get<{ marker_id: number }>(`${BASE_URL}/markers?name=${name}`, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      },
    });
    return response.data;
  } catch (error) {
    console.error('Error fetching marker by name:', error);
    return null;
  }
};

// 2. Create a new marker
export const create_marker = async (markerData: type.MarkerCreate): Promise<{ marker_id: number }> => {
  try {
    const response = await axios.post<{ marker_id: number }>(`${BASE_URL}/markers`, markerData, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error creating marker:', error);
    throw error;
  }
};

// 3. Update a marker
export const update_marker = async (marker_id: number, markerData: type.MarkerUpdate): Promise<{ status: string }> => {
  try {
    const response = await axios.put<{ status: string }>(`${BASE_URL}/markers/${marker_id}`, markerData, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error updating marker:', error);
    throw error;
  }
};

// 4. Delete a marker
export const delete_marker = async (marker_id: number): Promise<{ status: string }> => {
  try {
    const response = await axios.delete<{ status: string }>(`${BASE_URL}/markers/${marker_id}`, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error deleting marker:', error);
    throw error;
  }
};

// 5. Log a marker value
export const log_marker_value = async (marker_id: number, valueData: type.MarkerValue): Promise<{ status: string }> => {
  try {
    const response = await axios.post<{ status: string }>(`${BASE_URL}/markers/${marker_id}/logs`, valueData, {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error logging marker value:', error);
    throw error;
  }
};

// 6. Get marker analytics
export const get_marker_analytics = async (
  marker_id: number,
  from: string,
  to: string,
  metric: type.MetricType
): Promise<type.MarkerAnalyticsResponse | null> => {
  try {
    const response = await axios.get<type.MarkerAnalyticsResponse>(
      `${BASE_URL}/markers/${marker_id}/analytics?from=${from}&to=${to}&metric=${metric}`,
      {
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json'
        }
      }
    );
    return response.data;
  } catch (error) {
    console.error('Error fetching marker analytics:', error);
    return null;
  }
};

// 7. Get marker timeline
export const get_marker_timeline = async (
  marker_id: number,
  from: string,
  to: string
): Promise<type.TimelineEntry[]> => {
  try {
    const response = await axios.get<type.TimelineEntry[]>(
      `${BASE_URL}/markers/${marker_id}/timeline?from=${from}&to=${to}`,
      {
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json'
        }
      }
    );
    return response.data;
  } catch (error) {
    console.error('Error fetching marker timeline:', error);
    throw error;
  }
};