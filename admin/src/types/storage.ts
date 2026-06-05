import type { ListParamsReq } from './api'

export type StorageProvider = 'aliyun_oss' | 'cloudflare_r2'

export enum StorageChannelStatus {
  Disabled = 0,
  Enabled = 1,
}

export type StorageChannelConfig = {
  bucket: string
  region?: string | null
  endpoint: string
  access_key_id: string
  access_key_secret: string
  public_base_url: string
  prefix: string
  storage_class?: string | null
  object_acl?: string | null
}

export interface StorageChannel {
  id: number
  name: string
  provider: StorageProvider
  status: StorageChannelStatus
  sort_order: number
  config: StorageChannelConfig
  created_at: string
  updated_at: string
}

export type ListStorageChannelsParams = {
  id?: number
  provider?: StorageProvider
  status?: StorageChannelStatus
} & ListParamsReq

export interface SaveStorageChannelReq {
  name: string
  provider: StorageProvider
  status: StorageChannelStatus
  sort_order?: number
  config: StorageChannelConfig
}

export type VersionManifest = Record<string, unknown>

export type SyncVersionReq = {
  channel_ids?: number[]
}

export interface SyncVersionResult {
  channel_id: number
  channel_name: string
  provider: StorageProvider
  object_key: string
  public_url: string
  success: boolean
  status: number
  etag?: string | null
  error_message?: string | null
  log_id: number
}

export interface SyncVersionResp {
  app_id: number
  results: SyncVersionResult[]
}

export type VersionSyncLogStatus = 0 | 1 | 2

export interface VersionSyncLog {
  id: number
  app_id: number
  storage_channel_id: number
  provider: StorageProvider
  status: VersionSyncLogStatus
  object_key: string
  public_url: string
  manifest: VersionManifest
  error_message?: string | null
  etag?: string | null
  created_at: string
  finished_at?: string | null
}

export type ListVersionSyncLogsParams = {
  app_id?: number
  storage_channel_id?: number
  status?: VersionSyncLogStatus
  provider?: StorageProvider
} & ListParamsReq
