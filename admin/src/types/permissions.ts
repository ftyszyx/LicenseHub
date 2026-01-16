export interface PermissionInfo {
  id: number;
  name: string;
  resource: string;
  action: string;
  description?: string | null;
}

export interface RolePermissionIdsResp {
  role_id: number;
  permission_ids: number[];
}

export type SetRolePermissionsReq = {
  permission_ids: number[];
};