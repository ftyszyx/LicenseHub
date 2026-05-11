import type { ListParamsReq } from './api'
import type { RegCodeType } from './reg_codes'

export enum PlanStatus {
  Disabled = 0,
  Enabled = 1,
}

export enum OrderStatus {
  Pending = 0,
  Paid = 1,
  Delivered = 2,
  Failed = 3,
  Closed = 4,
}

export interface LicensePlan {
  id: number
  app_id: number
  app_name?: string | null
  name: string
  description?: string | null
  price_cents: number
  code_type: RegCodeType
  valid_days: number
  total_count?: number | null
  status: PlanStatus
  sort_order: number
  created_at: string
  updated_at: string
}

export type ListPlansParams = {
  id?: number
  app_id?: number
  name?: string
  status?: PlanStatus
} & ListParamsReq

export interface SavePlanReq {
  app_id: number
  name: string
  description?: string | null
  price_cents: number
  code_type: RegCodeType
  valid_days?: number | null
  total_count?: number | null
  status: PlanStatus
  sort_order?: number | null
}

export interface OrderModel {
  id: number
  order_no: string
  plan_id: number
  plan_name?: string | null
  app_id: number
  app_name?: string | null
  amount_cents: number
  pay_type: string
  status: OrderStatus
  provider: string
  provider_trade_no?: string | null
  pay_url?: string | null
  qr_code?: string | null
  url_scheme?: string | null
  reg_code_id?: number | null
  reg_code?: string | null
  paid_at?: string | null
  created_at: string
  updated_at: string
}

export type ListOrdersParams = {
  order_no?: string
  status?: OrderStatus
  plan_id?: number
  app_id?: number
} & ListParamsReq

export interface CreateOrderReq {
  plan_id: number
  pay_type: string
}

export interface PayMethodInfo {
  pay_type: string
  label: string
  enabled: boolean
}

export interface PayMethodsInfo {
  enabled: boolean
  provider: string
  merchant_active: boolean
  methods: PayMethodInfo[]
  message?: string | null
}
