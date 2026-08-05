import request from '@/utils/request'
import type { ApiResponse, PagingResponse } from '@/types'
import type {
  AdjustmentInfo,
  AdjustmentListParams,
  CommissionInfo,
  CommissionListParams,
  CreateSettlementReq,
  DistributionSummary,
  SettlementInfo,
  SettlementListParams,
} from '@/types/distribution'

export async function fetchMyDistributionSummary() {
  return ((await request.get('/admin/me/distribution/summary')) as ApiResponse<DistributionSummary>).data
}

export async function fetchMyCommissions(params: CommissionListParams = {}) {
  return ((await request.get('/admin/me/distribution/commissions', { params })) as ApiResponse<PagingResponse<CommissionInfo>>).data
}

export async function fetchMySettlements(params: SettlementListParams = {}) {
  return ((await request.get('/admin/me/distribution/settlements', { params })) as ApiResponse<PagingResponse<SettlementInfo>>).data
}

export async function fetchMyAdjustments(params: AdjustmentListParams = {}) {
  return ((await request.get('/admin/me/distribution/adjustments', { params })) as ApiResponse<PagingResponse<AdjustmentInfo>>).data
}

export async function requestWithdrawal(payload: CreateSettlementReq) {
  return ((await request.post('/admin/me/distribution/settlements', payload)) as ApiResponse<SettlementInfo>).data
}

export async function fetchAdminCommissions(params: CommissionListParams = {}) {
  return ((await request.get('/admin/distribution/commissions', { params })) as ApiResponse<PagingResponse<CommissionInfo>>).data
}

export async function fetchAdminSettlements(params: SettlementListParams = {}) {
  return ((await request.get('/admin/distribution/settlements', { params })) as ApiResponse<PagingResponse<SettlementInfo>>).data
}

export async function fetchAdminAdjustments(params: AdjustmentListParams = {}) {
  return ((await request.get('/admin/distribution/adjustments', { params })) as ApiResponse<PagingResponse<AdjustmentInfo>>).data
}

export async function rejectSettlement(id: number, reason: string) {
  return ((await request.post(`/admin/distribution/settlements/${id}/reject`, { reason })) as ApiResponse<SettlementInfo>).data
}

export async function markSettlementPaid(id: number, paymentReference: string, proof: File) {
  const form = new FormData()
  form.append('payment_reference', paymentReference)
  form.append('proof', proof)
  return ((await request.post(`/admin/distribution/settlements/${id}/paid`, form)) as ApiResponse<SettlementInfo>).data
}

export async function fetchSettlementProof(id: number, mine = false) {
  const path = mine
    ? `/admin/me/distribution/settlements/${id}/proof`
    : `/admin/distribution/settlements/${id}/proof`
  return await request.get(path, { responseType: 'blob' }) as unknown as Blob
}
