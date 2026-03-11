import type { ListParamsReq } from './api'

export interface UseRecordModel {
  id: number
  app_id: number
  app_name?: string | null
  device_id: string
  use_count: number
  use_info?: any | null
  time: number
}

export type ListUseRecordsParams = {
  app_id?: number
  device_id?: string
} & ListParamsReq
