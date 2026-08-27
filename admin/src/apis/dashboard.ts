import request from '@/utils/request'
import type { ApiResponse, DashboardStats, DashboardTrend, DashboardTrendParams } from '@/types'

export const fetchDashboardStats = async () => {
  const response = await request.get('/admin/dashboard') as ApiResponse<DashboardStats>
  return response.data
}

export const fetchDashboardTrend = async (params: DashboardTrendParams) => {
  const response = await request.get('/admin/dashboard/trend', { params }) as ApiResponse<DashboardTrend>
  return response.data
}
