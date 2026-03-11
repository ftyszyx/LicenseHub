import request from '@/utils/request'
import type { ApiResponse, PagingResponse } from '@/types'
import type { ListUseRecordsParams, UseRecordModel } from '@/types/use_records'

export const fetchUseRecords = async (params: ListUseRecordsParams = {}) => {
  const response = await request.get('/admin/use_records/list', { params }) as ApiResponse<PagingResponse<UseRecordModel>>
  return response.data
}
