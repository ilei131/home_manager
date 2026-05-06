import api from './client';
import type { SystemStats } from '../types';

export async function getSystemStats(): Promise<SystemStats> {
  const res = await api.get<SystemStats>('/stats/system');
  return res.data;
}

export async function getUserStats(): Promise<SystemStats> {
  const res = await api.get<SystemStats>('/stats/user');
  return res.data;
}
