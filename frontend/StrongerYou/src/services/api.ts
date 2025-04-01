import axios from 'axios';

const BASE_URL = 'https://localhost:8080/';

// Implementing APIs for routines.rs

// 1. Fetch RoutineID by RoutineName
export const get_routine_by_name = async (name : string) => {
  try {
    const response = await axios.get('${BASE_URL}/routines/name',
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