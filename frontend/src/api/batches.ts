import api from './client';
import type { Batch, CreateBatchRequest } from '../types';

export async function getBatches(itemId: string): Promise<Batch[]> {
  const res = await api.get<Batch[]>(`/items/${itemId}/batches`);
  return res.data;
}

export async function createBatch(itemId: string, data: CreateBatchRequest): Promise<Batch> {
  const res = await api.post<Batch>(`/items/${itemId}/batches`, data);
  return res.data;
}

export async function updateBatch(id: string, data: CreateBatchRequest): Promise<Batch> {
  const res = await api.put<Batch>(`/batches/${id}`, data);
  return res.data;
}

export async function deleteBatch(id: string): Promise<void> {
  await api.delete(`/batches/${id}`);
}
