import axios from 'axios';
import { getErrorMessage } from '../errors/errorCodes';

const api = axios.create({
    baseURL: '/api',
    timeout: 10000,
    headers: {
        'Content-Type': 'application/json',
    },
});

export interface ApiErrorResponse {
    code: string;
    message: string;
}

export class ApiError extends Error {
    public code: string;

    constructor(code: string, message: string) {
        super(message);
        this.code = code;
        this.name = 'ApiError';
    }
}

api.interceptors.request.use(
    (config) => {
        const token = localStorage.getItem('token');
        if (token) {
            config.headers.Authorization = `Bearer ${token}`;
        }
        return config;
    },
    (error) => Promise.reject(error)
);

api.interceptors.response.use(
    (response) => response,
    (error) => {
        if (error.response?.status === 401) {
            localStorage.removeItem('token');
            localStorage.removeItem('user');
            window.location.href = '/login';
        }

        if (error.response?.data) {
            const data = error.response.data as ApiErrorResponse;
            const errorCode = data.message || data.code;
            const friendlyMessage = getErrorMessage(errorCode);
            return Promise.reject(new ApiError(data.code || 'ERR_UNKNOWN', friendlyMessage));
        }

        return Promise.reject(error);
    }
);

export default api;
