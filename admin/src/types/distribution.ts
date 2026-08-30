import type { ListParamsReq } from './api'

export enum CommissionStatus {
  Pending = 0,
  Available = 1,
  Locked = 2,
  Settled = 3,
  Cancelled = 4,
  Offset = 5,
}

export enum SettlementStatus {
  Pending = 0,
  Paid = 1,
  Rejected = 2,
}

export interface AlipayAccountInfo {
  account: string
  real_name: string
}

export interface DistributionSummary {
  referral_code: string
  commission_rate_bps: number
  pending_amount_cents: number
  available_amount_cents: number
  locked_amount_cents: number
  settled_amount_cents: number
  adjustment_debt_cents: number
  min_withdraw_cents: number
  settlement_account?: AlipayAccountInfo | null
  order_count: number
  sales_amount_cents: number
}

export interface CommissionInfo {
  id: number
  order_id: number
  order_no: string
  order_time?: string | null
  user_id: number
  username?: string | null
  order_amount_cents: number
  commission_rate_bps: number
  commission_amount_cents: number
  available_amount_cents: number
  locked_amount_cents: number
  settled_amount_cents: number
  cancelled_amount_cents: number
  adjustment_amount_cents: number
  status: CommissionStatus
  available_at?: string | null
  created_at: string
}

export interface SettlementInfo {
  id: number
  settlement_no: string
  user_id: number
  username?: string | null
  amount_cents: number
  status: SettlementStatus
  settlement_account: AlipayAccountInfo
  payment_reference?: string | null
  payment_proof_file_name?: string | null
  payment_proof_content_type?: string | null
  payment_proof_size?: number | null
  reject_reason?: string | null
  requested_at: string
  reviewed_at?: string | null
  paid_at?: string | null
  reviewed_by?: number | null
  created_at: string
}

export interface AdjustmentInfo {
  id: number
  user_id: number
  username?: string | null
  order_id: number
  order_no: string
  original_commission_id: number
  amount_cents: number
  offset_amount_cents: number
  remaining_amount_cents: number
  reason: string
  status: number
  created_at: string
}

export interface CreateSettlementReq {
  amount_cents?: number
  alipay_account: string
  real_name: string
}

export type CommissionListParams = { status?: number; user_id?: number } & ListParamsReq
export type SettlementListParams = { status?: number; user_id?: number } & ListParamsReq
export type AdjustmentListParams = { status?: number; user_id?: number } & ListParamsReq
