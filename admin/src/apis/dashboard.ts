import request from '@/utils/request'
import type { ApiResponse, DashboardStats } from '@/types'

export const fetchDashboardStats = async () => {
  const response = await request.get('/admin/dashboard') as ApiResponse<DashboardStats>
  return response.data
}
