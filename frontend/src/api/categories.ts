import api from './client';
import type { Category } from '../types';

export async function getCategories(): Promise<Category[]> {
  const res = await api.get<Category[]>('/categories');
  return res.data;
}

export async function createCategory(name: string): Promise<Category> {
  const res = await api.post<Category>('/categories', { name });
  return res.data;
}

export async function updateCategory(id: number, name: string): Promise<Category> {
  const res = await api.put<Category>(`/categories/${id}`, { name });
  return res.data;
}

export async function deleteCategory(id: number): Promise<void> {
  await api.delete(`/categories/${id}`);
}
