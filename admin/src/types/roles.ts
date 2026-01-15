import type { ListParamsReq } from "./api";

export interface RoleInfo {
  id: number;
  name: string;
  description?: string | null;
  created_at: string;
  updated_at: string;
}
export type CreateRoleReq = { name: string; description?: string | null };
export type UpdateRoleReq = { name?: string; description?: string | null };
export type ListRolesParams = { id?: number; name?: string } & ListParamsReq;
