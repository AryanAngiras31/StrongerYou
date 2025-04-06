import axios from 'axios';

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
export const create_routine = async (routineData: { name: string, exercises: { exercise_id: number, sets: number }[] }) => {
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

// Exercises API

// 1. Fetch ExerciseID by ExerciseName
export const get_exercise_id_by_name = async (name: string): Promise<{ exerciseid: number }[]> => {
    try {
      const response = await axios.get(`${BASE_URL}/exercises`, {
        params: { name },
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
        },
      });
      return response.data;
    } catch (error) {
      console.error('Error fetching exercise ID by name:', error);
      throw error;
    }
  };
  
  // 2. Retrieve Exercise Details
  export const get_exercise_details = async (exerciseId: number): Promise<{ exerciseType: string; musclesTrained: string[] }> => {
    try {
      const response = await axios.get(`${BASE_URL}/exercises/${exerciseId}`, {
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
        },
      });
      return response.data;
    } catch (error) {
      console.error('Error fetching exercise details:', error);
      throw error;
    }
  };
  
  // 3. Create a New Exercise
  export const create_exercise = async (exerciseData: { exercise_name: string; muscles_trained: string[]; exercise_type: string }): Promise<{ exerciseid: number; exercisename: string; muscles_trained: string[]; exercisetype: string }> => {
    try {
      const response = await axios.post(`${BASE_URL}/exercises`, exerciseData, {
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
        },
      });
      return response.data;
    } catch (error) {
      console.error('Error creating exercise:', error);
      throw error;
    }
  };
  
  // 4. Modify Exercise
  export const update_exercise = async (exerciseId: number, exerciseData: { exercise_name: string; muscles_trained: string[]; exercise_type: string }): Promise<any> => {
    try {
      const response = await axios.put(`${BASE_URL}/exercises/${exerciseId}`, exerciseData, {
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
        },
      });
      return response.data;
    } catch (error) {
      console.error('Error updating exercise:', error);
      throw error;
    }
  };
  
  // 5. Delete an Exercise
  export const delete_exercise = async (exerciseId: number): Promise<{ exerciseid: number }> => {
    try {
      const response = await axios.delete(`${BASE_URL}/exercises/${exerciseId}`, {
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
        },
      });
      return response.data;
    } catch (error) {
      console.error('Error deleting exercise:', error);
      throw error;
    }
  };
  
  // 6. Return Personal Records for the Exercise
  export const get_exercise_prs = async (exerciseId: number): Promise<{ workout_date: string; weight: number; one_rm: number; set_volume: number }[]> => {
    try {
      const response = await axios.get(`${BASE_URL}/exercises/${exerciseId}/prs`, {
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
        },
      });
      return response.data;
    } catch (error) {
      console.error('Error fetching exercise PRs:', error);
      throw error;
    }
  };
  
  // 7. Return Progress Graph for the Exercise
  export const get_exercise_progress = async (
    exerciseId: number,
    metric: 'HeaviestWeight' | 'SetVolume' | 'OneRM',
    from?: string,
    to?: string
  ): Promise<{ date: string; value: number }[]> => {
    try {
      const response = await axios.get(`${BASE_URL}/exercises/${exerciseId}/progress`, {
        params: { metric, from, to },
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
        },
      });
      return response.data;
    } catch (error) {
      console.error(`Error fetching ${metric} progress for exercise ${exerciseId}:`, error);
      throw error;
    }
  };