import type { ListParamsReq } from './api'

export interface RegCodeModel {
  id: number
  code: string
  app_id: number
  bind_device_info?: any | null
  valid_days: number
  max_devices: number
  status: RegCodeStatus
  binding_time?: string | null
  code_type: RegCodeType
  total_count?: number | null
  remaining_count?: number | null
  device_id?: number | null
  device_id_str?: string | null
  device_ids?: string[]
  bound_device_count?: number
  created_at: string
  updated_at: string
  app_name?: string | null
}

export enum RegCodeType {
  Time = 0,
  Count = 1,
}

export enum RegCodeStatus {
  Unused = 0,
  Issued = 1,
  binded = 2,
  Refunded = 3,
  Revoked = 4,
}

export type ListRegCodesParams = {
  id?: number
  code?: string
  app_id?: number
  status?: RegCodeStatus
  code_type?: RegCodeType
  device_id?: string
} & ListParamsReq

export interface BatchCreateRegCodesReq {
  app_id: number
  quantity: number
  code_type: RegCodeType
  valid_days?: number | null
  total_count?: number | null
}

export interface UpdateRegCodeReq {
  max_devices: number
  remaining_count?: number
}

export interface BindRegCodeReq {
  app_key: string
  reg_code: string
  device_id: string
}

export interface CheckRegDeviceReq {
  app_key: string
  device_id: string
}

export interface RegCodeBindCheckResp {
  expire_time?: number | null
  remain_count?: number | null
}

export interface UseCountReq {
  app_key: string
  device_id: string
  use_count: number
  use_info?: any | null
}

export interface UseCountResp {
  remain_count: number
}

