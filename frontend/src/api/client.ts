import axios from 'axios';

const API_URL = 'http://localhost:5000';

export const client = axios.create({
    baseURL: API_URL,
    headers: {
        'Content-Type': 'application/json',
    },
});

client.interceptors.request.use((config) => {
    const token = localStorage.getItem('token');
    if (token) {
        config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
});

// Auth API
export const auth = {
    login: async (email: string, password: string) => {
        const response = await client.post('/auth/login', { email, password });
        const { token } = response.data;
        localStorage.setItem('token', token);
        return response.data;
    },
    register: async (username: string, email: string, password: string) => {
        const response = await client.post('/auth/register', { username, email, password });
        const { token } = response.data;
        localStorage.setItem('token', token);
        return response.data;
    },
    logout: () => {
        localStorage.removeItem('token');
    },
};

// Media API
export const media = {
    upload: async (file: File) => {
        const formData = new FormData();
        formData.append('file', file);
        const response = await client.post('/media', formData, {
            headers: {
                'Content-Type': 'multipart/form-data',
            },
        });
        return response.data;
    },
    list: async () => {
        const response = await client.get('/media');
        return response.data;
    },
    get: async (id: string) => {
        const response = await client.get(`/media/${id}`, {
            responseType: 'blob',
        });
        return response.data;
    },
    delete: async (id: string) => {
        await client.delete(`/media/${id}`);
    },
}; 