import api from './client';
import type { Item, CreateItemRequest } from '../types';

export async function getItems(): Promise<Item[]> {
  const res = await api.get<Item[]>('/items');
  return res.data;
}

export async function getItem(id: string): Promise<Item> {
  const res = await api.get<Item>(`/items/${id}`);
  return res.data;
}

export async function createItem(data: CreateItemRequest): Promise<Item> {
  const res = await api.post<Item>('/items', data);
  return res.data;
}

export async function updateItem(id: string, data: CreateItemRequest): Promise<Item> {
  const res = await api.put<Item>(`/items/${id}`, data);
  return res.data;
}

export async function deleteItem(id: string): Promise<void> {
  await api.delete(`/items/${id}`);
}

export async function getExpiringItems(): Promise<Item[]> {
  const res = await api.get<Item[]>('/items/expiring');
  return res.data;
}
