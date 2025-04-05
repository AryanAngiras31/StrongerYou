import axios from 'axios';

const BASE_URL = 'https://localhost:8080/';

// Helper function to include logging in each API call
const makeRequest = async (method: 'get' | 'post' | 'put' | 'delete', url: string, data?: any, params?: any) => {
    console.log(`[${method.toUpperCase()}] Request to: ${url}`, { data, params }); // Log request details

    try {
        const response = await axios({
            method,
            url,
            data,
            params,
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            }
        });
        console.log(`[${method.toUpperCase()}] Response from: ${url}`, { status: response.status, data: response.data }); // Log response
        return response.data;
    } catch (error: any) {
        console.error(`[${method.toUpperCase()}] Error from: ${url}`, { error }); // Log error
        if (error.response) {
            // The request was made and the server responded with a status code
            // that falls out of the range of 2xx
            console.error("Response Data:", error.response.data);
            console.error("Response Status:", error.response.status);
            console.error("Response Headers:", error.response.headers);
        } else if (error.request) {
            // The request was made but no response was received
            console.error("Request Error:", error.request);
        } else {
            // Something happened in setting up the request that triggered an Error
            console.error("Error Message:", error.message);
        }
        throw error; // Re-throw the error to be caught by the caller
    }
};

// Routines API
// 1. Fetch RoutineID by RoutineName
export const get_routine_by_name = async (name: string) => {
    return makeRequest('get', `${BASE_URL}/routines/name`, undefined, { name });
};

// 2. Create Routine
export const create_routine = async (routineData: { name: string, exercises: { exercise_id: number, sets: number }[] }) => {
    return makeRequest('post', `${BASE_URL}/routines`, routineData);
};

// 3. Modify Routine
export const update_routine = async (routineId: number, routineData: { name: string, exercises: { exercise_id: number, sets: number }[] }) => {
    return makeRequest('put', `${BASE_URL}/routines/${routineId}`, routineData);
};

// 4. Delete Routine
export const delete_routine = async (routineId: number) => {
    return makeRequest('delete', `${BASE_URL}/routines/${routineId}`);
};

// 5. Display Routines
export const list_routines = async (includeLastPerformed: boolean = false) => {
    return makeRequest('get', `${BASE_URL}/routines`, undefined, {
        include: includeLastPerformed ? 'lastPerformed' : undefined,
    });
};

// 6. View Routine
export const view_routine = async (routineId: number) => {
    return makeRequest('get', `${BASE_URL}/routines/${routineId}`);
};
