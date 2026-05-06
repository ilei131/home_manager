import api from './client';
import type { Location } from '../types';

export async function getLocations(): Promise<Location[]> {
  const res = await api.get<Location[]>('/locations');
  return res.data;
}

export async function createLocation(name: string): Promise<Location> {
  const res = await api.post<Location>('/locations', { name });
  return res.data;
}

export async function updateLocation(id: number, name: string): Promise<Location> {
  const res = await api.put<Location>(`/locations/${id}`, { name });
  return res.data;
}

export async function deleteLocation(id: number): Promise<void> {
  await api.delete(`/locations/${id}`);
}
