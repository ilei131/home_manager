export interface User {
  id: string;
  username: string;
  role: 'admin' | 'user';
  created_at: string;
}

export interface Category {
  id: number;
  user_id: string | null;
  name: string;
  is_system: boolean;
  created_at: string;
}

export interface Location {
  id: number;
  user_id: string | null;
  name: string;
  is_system: boolean;
  created_at: string;
}

export interface Batch {
  id: string;
  item_id: string;
  quantity: number;
  expiry_date: string | null;
  created_at: string;
}

export interface Item {
  id: string;
  user_id: string;
  name: string;
  category_id: number;
  location_id: number;
  category_name: string;
  location_name: string;
  batches: Batch[];
  created_at: string;
  updated_at: string;
}

export interface SystemStats {
  total_users: number;
  total_items: number;
  total_categories: number;
  total_locations: number;
  total_batches: number;
}

export interface CreateItemRequest {
  name: string;
  category_id: number;
  location_id: number;
  quantity?: number;
  expiry_date?: string | null;
}

export interface CreateBatchRequest {
  quantity: number;
  expiry_date: string | null;
}

export interface LoginRequest {
  username: string;
  password: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}
