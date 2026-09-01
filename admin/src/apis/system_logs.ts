import request from '@/utils/request'
import type { ApiResponse } from '@/types'
import type { ListSystemLogsParams, SystemLogPage } from '@/types/system_logs'

export const fetchSystemLogs = async (params: ListSystemLogsParams = {}) => {
  const response = await request.get('/admin/system-logs', { params }) as ApiResponse<SystemLogPage>
  return response.data
}
