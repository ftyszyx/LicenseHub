import request from '@/utils/request'
import type { ApiResponse, GenerateLicenseSigningKeyReq, SaveSystemSettingsReq, SiteSettings } from '@/types'

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

export const generateLicenseSigningKey = async (payload: GenerateLicenseSigningKeyReq) => {
  const response = await request.post('/admin/system-settings/license-key', payload) as ApiResponse<SiteSettings>
  return response.data
}

export const sendSystemTestEmail = async (email: string) => {
  const response = await request.post('/admin/system-settings/test-email', { email }) as ApiResponse<boolean>
  return response.data
}
