import axios from 'axios';
import * as type from '../services/types';
const BASE_URL = 'https://localhost:8080';

// Routines API

// 1. Fetch RoutineID by RoutineName
export const get_routine_by_name = async (name : string) => {
  try {
    const response = await axios.get(`${BASE_URL}/routines/name`,
        {
            params: { name },
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            },
        }
    );
    return response.data;
  }
  catch (error) {
    console.error('Error fetching routine by name:', error);
    throw error;
  }
};

// 2. Create Routine
export const create_routine = async (routineData: type.RoutineCreate) => {
  try {
      const response = await axios.post(`${BASE_URL}/routines`, routineData, {
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
export const update_routine = async (routineId: number, routineData: { name: string, exercises: { exercise_id: number, sets: number }[] }) => {
  try {
      const response = await axios.put(`${BASE_URL}/routines/${routineId}`, routineData, {
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
export const delete_routine = async (routineId: number) => {
  try {
      const response = await axios.delete(`${BASE_URL}/routines/${routineId}`, {
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
export const list_routines = async (includeLastPerformed: boolean = false) => {
  try {
      const response = await axios.get(`${BASE_URL}/routines`, {
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
export const view_routine = async (routineId: number) => {
  try {
      const response = await axios.get(`${BASE_URL}/routines/${routineId}`, {
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
export const start_a_new_workout = async (routineId: number) => {
  try {
    const response = await axios.get(`${BASE_URL}/workouts/template/${routineId}`, {
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json'
        }
    });
    return response.data;
  } catch (error) {
    console.error('Error starting a new workout:', error);
  }
};


// 2. Modify an Existing Workout 
export const modify_workout = async (workoutId: number, workout_data: {routine_id: number, start_time: string, end_time: string, exercises: {exercise_id: number, exercise_name: string,  sets: {set_number:number, set:{weight: number, reps: number}[]}[]}[]}) => {
    try {
        const response = await axios.put(`${BASE_URL}/workouts/${workoutId}`, {
            params: workout_data,
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            }
        });
        return response.data;
    } catch (error) {   
        console.error('Error modifying workout:', error);
    }
}

// 3. Finish Workout
export const finish_workout = async (workout_data: {routine_id: number, start_time: string, end_time: string, exercises: {exercise_id: number, exercise_name: string,  sets: {set_number:number, set:{weight: number, reps: number}[]}[]}[]}) => {
    try {
        const response = await axios.post(`${BASE_URL}/workouts`, {
            params: workout_data,
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            }
        });
        return response.data;
    } catch (error) {
        console.error('Error finishing workout:', error);
    }
}

// 4. Validate Set
export const validate_set = async (exercise_id: number, weight: number, reps: number) => {
    try {
        const response = await axios.post(`${BASE_URL}/workouts/validate`, {
            params: {exercise_id, weight, reps},
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            }
        });
        return response.data;
    } catch (error) {
        console.error('Error validating set:', error);
    }
}

// 5. Display Workouts
export const display_workouts = async () => {
    try {
        const response = await axios.get(`${BASE_URL}/workouts`, {
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            }
        });
        return response.data;
    } catch (error) {
        console.error('Error displaying workouts:', error);
    }
}

// 6. View Workout
export const view_workout = async (workoutId: number) => {
    try {
        const response = await axios.get(`${BASE_URL}/workouts/${workoutId}`, {
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            }
        });
        return response.data;
    } catch (error) {
        console.error('Error viewing workout:', error);
    }
}
