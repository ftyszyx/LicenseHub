import request from '@/utils/request'
import type { ApiResponse, PagingResponse } from '@/types'
import type {
  ListStorageChannelsParams,
  ListVersionSyncLogsParams,
  SaveStorageChannelReq,
  StorageChannel,
  SyncVersionReq,
  SyncVersionResp,
  VersionManifest,
  VersionSyncLog,
} from '@/types/storage'

export const fetchStorageChannels = async (params: ListStorageChannelsParams = {}) => {
  const response = await request.get('/admin/storage-channels/list', { params }) as ApiResponse<PagingResponse<StorageChannel>>
  return response.data
}

export const createStorageChannel = async (payload: SaveStorageChannelReq) => {
  const response = await request.post('/admin/storage-channels', payload) as ApiResponse<StorageChannel>
  return response.data
}

export const updateStorageChannel = async (id: number, payload: Partial<SaveStorageChannelReq>) => {
  const response = await request.put(`/admin/storage-channels/${id}`, payload) as ApiResponse<StorageChannel>
  return response.data
}

export const deleteStorageChannel = async (id: number) => {
  const response = await request.delete(`/admin/storage-channels/${id}`) as ApiResponse<void>
  return response.data
}

export const fetchVersionManifest = async (id: number) => {
  const response = await request.get(`/admin/apps/${id}/version-manifest`) as ApiResponse<VersionManifest>
  return response.data
}

export const syncAppVersion = async (id: number, payload: SyncVersionReq = {}) => {
  const response = await request.post(`/admin/apps/${id}/sync-version`, payload) as ApiResponse<SyncVersionResp>
  return response.data
}

export const fetchVersionSyncLogs = async (params: ListVersionSyncLogsParams = {}) => {
  const response = await request.get('/admin/version-sync-logs', { params }) as ApiResponse<PagingResponse<VersionSyncLog>>
  return response.data
}
