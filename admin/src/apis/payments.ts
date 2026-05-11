import request from '@/utils/request'
import type { ApiResponse, PagingResponse } from '@/types'
import type {
  CreateOrderReq,
  LicensePlan,
  ListOrdersParams,
  ListPlansParams,
  OrderModel,
  PayMethodsInfo,
  SavePlanReq,
} from '@/types/payments'

export const fetchPublicPlans = async (params: { app_id?: number } = {}) => {
  const response = await request.get('/products', { params }) as ApiResponse<LicensePlan[]>
  return response.data
}

export const fetchPayMethods = async () => {
  const response = await request.get('/pay/methods') as ApiResponse<PayMethodsInfo>
  return response.data
}

export const createOrder = async (payload: CreateOrderReq) => {
  const response = await request.post('/orders', payload) as ApiResponse<OrderModel>
  return response.data
}

export const fetchOrder = async (orderNo: string) => {
  const response = await request.get(`/orders/${orderNo}`) as ApiResponse<OrderModel>
  return response.data
}

export const fetchPlans = async (params: ListPlansParams = {}) => {
  const response = await request.get('/admin/plans/list', { params }) as ApiResponse<PagingResponse<LicensePlan>>
  return response.data
}

export const createPlan = async (payload: SavePlanReq) => {
  const response = await request.post('/admin/plans', payload) as ApiResponse<LicensePlan>
  return response.data
}

export const updatePlan = async (id: number, payload: Partial<SavePlanReq>) => {
  const response = await request.put(`/admin/plans/${id}`, payload) as ApiResponse<LicensePlan>
  return response.data
}

export const deletePlan = async (id: number) => {
  const response = await request.delete(`/admin/plans/${id}`) as ApiResponse<void>
  return response.data
}

export const fetchOrders = async (params: ListOrdersParams = {}) => {
  const response = await request.get('/admin/orders/list', { params }) as ApiResponse<PagingResponse<OrderModel>>
  return response.data
}
