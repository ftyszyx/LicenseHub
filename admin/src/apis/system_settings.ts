import request from '@/utils/request'
import type { ApiResponse, SaveSystemSettingsReq, SiteSettings } from '@/types'

export const fetchSiteSettings = async () => {
  const response = await request.get('/site-settings') as ApiResponse<SiteSettings>
  return response.data
}

export const fetchSystemSettings = async () => {
  const response = await request.get('/admin/system-settings') as ApiResponse<SiteSettings>
  return response.data
}

export const updateSystemSettings = async (payload: SaveSystemSettingsReq) => {
  const response = await request.put('/admin/system-settings', payload) as ApiResponse<SiteSettings>
  return response.data
}
