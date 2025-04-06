import {
    get_routine_by_name,
    create_routine,
    update_routine,
    delete_routine,
    list_routines,
    view_routine
} from '../services/api'; // Adjust the path if needed
import axios from 'axios';
import { vi, describe, it, expect, beforeEach } from 'vitest';

// Mock axios
vi.mock('axios');
const mockedAxios = axios as vi.Mocked<typeof axios>;

// Define reusable headers
const standardHeaders = {
    'Accept': 'application/json',
    'Content-Type': 'application/json'
};

// Define Base URL used in api.ts
const BASE_URL = 'https://localhost:8080';

describe('Routines API Tests', () => {
    let createdRoutineId: number | null = null; // To store the ID across tests
    const initialRoutineName = 'Morning Workout';
    // Frontend representation for create/update
    const initialRoutineExercises = [
        { exercise_id: 1, sets: 3 },
        { exercise_id: 5, sets: 4 },
    ];
    const initialRoutineData = {
        name: initialRoutineName,
        exercises: initialRoutineExercises
    };
    // Backend representation for view response (matches ExerciseSetPair)
    const viewRoutineExercises = [
        { exercise_id: 1, num_sets: 3 },
        { exercise_id: 5, num_sets: 4 },
    ];


    beforeEach(() => {
        // Reset mocks before each test to avoid interference
        vi.clearAllMocks();
        // Reset ID for isolated test runs if needed, but the flow depends on it persisting
        // createdRoutineId = null;
    });

    // 1. Create a New Routine (POST /routines)
    // Rust Response: { "routine_id": number }
    it('should create a new routine', async () => {
        const mockBackendResponse = {
            routine_id: 101, // Example ID returned by the backend
        };
        mockedAxios.post.mockResolvedValue({ data: mockBackendResponse });

        const result = await create_routine(initialRoutineData);

        expect(mockedAxios.post).toHaveBeenCalledWith(`${BASE_URL}/routines`, initialRoutineData, {
            headers: standardHeaders
        });
        // The frontend api.ts function likely just returns response.data
        expect(result).toEqual(mockBackendResponse);

        // Store the ID for subsequent tests
        createdRoutineId = mockBackendResponse.routine_id;
        console.log(`Created routine with ID: ${createdRoutineId}`); // Log for clarity
    });

    // 2. Fetch RoutineID by RoutineName (GET /routines/name)
    // Rust Response: { "routine_id": number }
    it('should fetch routine ID by routine name', async () => {
        // Ensure the previous test ran and set the ID
        expect(createdRoutineId).not.toBeNull();

        const mockBackendResponse = {
            routine_id: createdRoutineId, // Expecting only the ID back
        };
        mockedAxios.get.mockResolvedValue({ data: mockBackendResponse });

        const result = await get_routine_by_name(initialRoutineName);

        expect(mockedAxios.get).toHaveBeenCalledWith(`${BASE_URL}/routines/name`, {
            params: { name: initialRoutineName },
            headers: standardHeaders
        });
        expect(result).toEqual(mockBackendResponse);
        // Verify the fetched ID matches the one we created
        expect(result.routine_id).toEqual(createdRoutineId);
        console.log(`Workspaceed routine by name, confirmed ID: ${result.routine_id}`);
    });

    // 3. View Routine Details (GET /routines/{routine_id})
    // Rust Response: RoutineViewResponse { routine_id, routine_name, routines: Vec<ExerciseSetPair> }
    it('should retrieve routine details using the fetched ID', async () => {
        expect(createdRoutineId).not.toBeNull(); // Ensure ID exists

        const mockBackendResponse = {
            routine_id: createdRoutineId,
            routine_name: initialRoutineName,
            routines: viewRoutineExercises // Using the backend structure {exercise_id, num_sets}
        };
        mockedAxios.get.mockResolvedValue({ data: mockBackendResponse });

        const result = await view_routine(createdRoutineId!); // Use non-null assertion

        expect(mockedAxios.get).toHaveBeenCalledWith(`${BASE_URL}/routines/${createdRoutineId}`, {
            headers: standardHeaders
        });
        expect(result).toEqual(mockBackendResponse);
        console.log(`Viewed details for routine ID: ${createdRoutineId}`);
    });

    // 4. Display Routines (List) (GET /routines)
    // Rust Response: Vec<RoutineInfo { routine_id, name, timestamp, last_performed? }>
    it('should list routines and contain the created routine', async () => {
       expect(createdRoutineId).not.toBeNull(); // Ensure ID exists

        // Since we assume the table was empty, the list should only contain our routine
        const mockBackendResponse = [
            {
                routine_id: createdRoutineId,
                name: initialRoutineName,
                timestamp: "2025-04-05T10:00:00Z", // Example ISO timestamp (needs to be valid format if parsed)
                last_performed: null // Assuming include=lastPerformed is false
            }
        ];
        mockedAxios.get.mockResolvedValue({ data: mockBackendResponse });

        const result = await list_routines(); // Call without includeLastPerformed

        expect(mockedAxios.get).toHaveBeenCalledWith(`${BASE_URL}/routines`, {
            params: { include: undefined }, // Expecting default params
            headers: standardHeaders
        });
        expect(result).toEqual(mockBackendResponse);
        expect(result).toHaveLength(1); // Check if only one routine is listed
        expect(result[0].routine_id).toEqual(createdRoutineId); // Verify the ID
        expect(result[0].name).toEqual(initialRoutineName);
        expect(result[0]).toHaveProperty('timestamp');
        expect(result[0].last_performed).toBeNull();
        console.log(`Listed routines, found created routine ID: ${createdRoutineId}`);
    });

     // 5. Modify Routine (PUT /routines/{routine_id})
     // Rust Response: { "status": "updated" }
    it('should modify the existing routine', async () => {
        expect(createdRoutineId).not.toBeNull(); // Ensure ID exists

        // Frontend representation for update
        const updatedRoutineExercises = [
            { exercise_id: 1, sets: 4 }, // Increased sets for exercise 1
            { exercise_id: 5, sets: 4 },
            { exercise_id: 8, sets: 3 }, // Added a new exercise
        ];
        const updatedRoutineData = {
            name: 'Morning Power Workout', // Changed name
            exercises: updatedRoutineExercises
        };
        const mockBackendResponse = {
            status: 'updated'
        };
        mockedAxios.put.mockResolvedValue({ data: mockBackendResponse });

        const result = await update_routine(createdRoutineId!, updatedRoutineData); // Use non-null assertion

        expect(mockedAxios.put).toHaveBeenCalledWith(`${BASE_URL}/routines/${createdRoutineId}`, updatedRoutineData, {
            headers: standardHeaders
        });
        expect(result).toEqual(mockBackendResponse);
        console.log(`Updated routine ID: ${createdRoutineId}`);
    });

     // 6. Delete Routine (DELETE /routines/{routine_id})
     // Rust Response: { "status": "deleted" }
    it('should delete the routine', async () => {
        expect(createdRoutineId).not.toBeNull(); // Ensure ID exists

        const mockBackendResponse = {
            status: 'deleted'
        };
        mockedAxios.delete.mockResolvedValue({ data: mockBackendResponse });

        const result = await delete_routine(createdRoutineId!); // Use non-null assertion

        expect(mockedAxios.delete).toHaveBeenCalledWith(`${BASE_URL}/routines/${createdRoutineId}`, {
            headers: standardHeaders
        });
        expect(result).toEqual(mockBackendResponse);
        console.log(`Deleted routine ID: ${createdRoutineId}`);
    });

    // Optional: Add a test to list routines again to confirm deletion
    // Rust Response: [] (empty Vec<RoutineInfo>)
    it('should list routines and confirm the routine is deleted', async () => {
        // This test assumes the delete was successful in the previous step's context
        const mockBackendResponse: any[] = []; // Expect an empty list now
        mockedAxios.get.mockResolvedValue({ data: mockBackendResponse });

        const result = await list_routines();

        expect(mockedAxios.get).toHaveBeenCalledWith(`${BASE_URL}/routines`, {
                params: { include: undefined },
                headers: standardHeaders
        });
        expect(result).toEqual([]);
        expect(result).toHaveLength(0);
        console.log(`Confirmed routine deletion by listing routines (empty list).`);
    });

});