import request from '@/utils/request'
import type { ApiResponse, PagingResponse } from '@/types'
import type { ListResourcesParams, ResourceModel } from '@/types/resources'

export const fetchResources = async (params: ListResourcesParams = {}) => {
  const response = await request.get('/admin/resources/list', { params }) as ApiResponse<PagingResponse<ResourceModel>>
  return response.data
}

export const uploadResource = async (resourceType: string, file: File) => {
  const form = new FormData()
  form.append('resource_type', resourceType)
  form.append('file', file)
  const response = await request.post('/admin/resources', form) as ApiResponse<ResourceModel>
  return response.data
}

export const fetchResourceBlob = async (id: number) => {
  return await request.get(`/admin/resources/${id}/download`, { responseType: 'blob' }) as unknown as Blob
}

export const deleteResource = async (id: number) => {
  const response = await request.delete(`/admin/resources/${id}`) as ApiResponse<void>
  return response.data
}
