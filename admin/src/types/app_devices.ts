import type { ListParamsReq } from "./api"
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
}
export type ListDevicesParams = {
  app_id?: number
  device_id?: string
} & ListParamsReq
