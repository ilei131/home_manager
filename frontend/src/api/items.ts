import api from './client';
import type { Item, CreateItemRequest } from '../types';

export interface PaginatedItemsResponse {
  items: Item[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

export async function getItems(page = 1, pageSize = 10): Promise<PaginatedItemsResponse> {
  const res = await api.get<PaginatedItemsResponse>('/items', {
    params: { page, page_size: pageSize },
  });
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
