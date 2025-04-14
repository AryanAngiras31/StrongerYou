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
export const start_a_new_workout = async (routineId: number): Promise<{ exercises: type.Exercise[] }> => {
  try {
    const response = await axios.get<{ exercises: type.Exercise[] }>(`${BASE_URL}/workouts/template/${routineId}`, {
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