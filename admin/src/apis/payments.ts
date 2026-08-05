import request from '@/utils/request'
import type { ApiResponse, PagingResponse } from '@/types'
import type {
  ConfirmOrderRefundReq,
  CreateOrderReq,
  LicensePlan,
  ListOrdersParams,
  ListPaymentChannelsParams,
  ListPlansParams,
  OrderModel,
  PaymentChannel,
  PayMethodsInfo,
  PublicPlansInfo,
  SavePaymentChannelReq,
  SavePlanReq,
} from '@/types/payments'

export const fetchPublicPlans = async (params: { app_id?: number } = {}) => {
  const response = await request.get('/products', { params }) as ApiResponse<LicensePlan[] | PublicPlansInfo>
  if (Array.isArray(response.data)) {
    return {
      state: 'available',
      app_id: params.app_id ?? null,
      app_name: null,
      app_status: null,
      plans: response.data,
    } satisfies PublicPlansInfo
  }
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

export const confirmOrderRefund = async (id: number, payload: ConfirmOrderRefundReq) => {
  const response = await request.post(`/admin/orders/${id}/refund`, payload) as ApiResponse<OrderModel>
  return response.data
}

export const fetchPaymentChannels = async (params: ListPaymentChannelsParams = {}) => {
  const response = await request.get('/admin/payment-channels/list', { params }) as ApiResponse<PagingResponse<PaymentChannel>>
  return response.data
}

export const createPaymentChannel = async (payload: SavePaymentChannelReq) => {
  const response = await request.post('/admin/payment-channels', payload) as ApiResponse<PaymentChannel>
  return response.data
}

export const updatePaymentChannel = async (id: number, payload: Partial<SavePaymentChannelReq>) => {
  const response = await request.put(`/admin/payment-channels/${id}`, payload) as ApiResponse<PaymentChannel>
  return response.data
}

export const deletePaymentChannel = async (id: number) => {
  const response = await request.delete(`/admin/payment-channels/${id}`) as ApiResponse<void>
  return response.data
}
