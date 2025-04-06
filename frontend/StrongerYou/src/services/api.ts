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

