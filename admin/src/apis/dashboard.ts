import request from '@/utils/request'
import type {
  ApiResponse,
  DashboardReturningOrder,
  DashboardReturningOrdersParams,
  DashboardStats,
  DashboardTrend,
  DashboardTrendParams,
  PagingResponse,
} from '@/types'

export const fetchDashboardStats = async () => {
  const response = await request.get('/admin/dashboard') as ApiResponse<DashboardStats>
  return response.data
}

export const fetchDashboardTrend = async (params: DashboardTrendParams) => {
  const response = await request.get('/admin/dashboard/trend', { params }) as ApiResponse<DashboardTrend>
  return response.data
}

export const fetchDashboardReturningOrders = async (params: DashboardReturningOrdersParams) => {
  const response = await request.get('/admin/dashboard/returning-orders', { params }) as ApiResponse<PagingResponse<DashboardReturningOrder>>
  return response.data
}
