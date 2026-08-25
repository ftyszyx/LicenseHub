import type { ListParamsReq } from './api'

export interface ResourceModel {
  id: number
  storage_channel_id: number
  storage_channel_name?: string | null
  resource_type: string
  original_name: string
  content_type: string
  size: number
  uploaded_by: number
  created_at: string
  updated_at: string
}

export type ListResourcesParams = {
  resource_type?: string
  keyword?: string
} & ListParamsReq
