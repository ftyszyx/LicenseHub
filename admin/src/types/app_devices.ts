import type { ListParamsReq } from "./api"

export interface DeviceRegCodeInfo {
  id: number
  code: string
  status: number
  code_type: number
  binding_time: string
  effective_time?: string | null
  expire_time?: string | null
}

export interface DeviceInfo {
  id: number
  app_id: number
  app_name: string
  device_id: string
  device_info?: any | null
  bind_time?: string | null
  expire_time?: string | null
  created_at?: string | null
  updated_at?: string | null
  remaining?: number | null
  bound_reg_code_count: number
  reg_codes: DeviceRegCodeInfo[]
}
export type ListDevicesParams = {
  app_id?: number
  device_id?: string
  sort_by?: 'created_at' | 'bound_reg_code_count'
  sort_order?: 'asc' | 'desc'
} & ListParamsReq
