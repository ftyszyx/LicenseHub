import request from "@/utils/request";
import type { ApiResponse } from "@/types";
import type { PermissionInfo, RolePermissionIdsResp, SetRolePermissionsReq } from "@/types";

export async function fetchPermissions(): Promise<PermissionInfo[]> {
  return (await request.get("/admin/permissions/list") as ApiResponse<PermissionInfo[]>).data;
}

export async function fetchRolePermissionIds(roleId: number): Promise<RolePermissionIdsResp> {
  return (await request.get(`/admin/roles/${roleId}/permissions`) as ApiResponse<RolePermissionIdsResp>).data;
}

export async function setRolePermissions(roleId: number, payload: SetRolePermissionsReq): Promise<boolean> {
  return (await request.post(`/admin/roles/${roleId}/permissions`, payload) as ApiResponse<boolean>).data;
}
