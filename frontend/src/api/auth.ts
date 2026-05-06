import api from './client';
import type { LoginRequest, AuthResponse, User } from '../types';

export async function login(data: LoginRequest): Promise<AuthResponse> {
  const res = await api.post<AuthResponse>('/auth/login', data);
  return res.data;
}

export async function register(data: LoginRequest): Promise<AuthResponse> {
  const res = await api.post<AuthResponse>('/auth/register', data);
  return res.data;
}

export async function getMe(): Promise<User> {
  const res = await api.get<User>('/auth/me');
  return res.data;
}
