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
  Refunded = 5,
}

export interface OrderRefundInfo {
  refund_no: string
  refund_reference: string
  reason: string
  operator_user_id: number
  refunded_at: string
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

export type PublicPlansState = 'available' | 'app_disabled' | 'app_not_found'

export interface PublicPlansInfo {
  state: PublicPlansState
  app_id?: number | null
  app_name?: string | null
  app_status?: number | null
  plans: LicensePlan[]
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
  refund?: OrderRefundInfo | null
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
  referral_code?: string
}

export interface ConfirmOrderRefundReq {
  refund_reference: string
  reason: string
}

export interface PayMethodInfo {
  pay_type: string
  label: string
  provider: string
  enabled: boolean
}

export interface PayMethodsInfo {
  enabled: boolean
  provider: string
  merchant_active: boolean
  methods: PayMethodInfo[]
  message?: string | null
}

export type PaymentProvider = 'wechat' | 'alipay'

export enum PaymentChannelStatus {
  Disabled = 0,
  Enabled = 1,
}

export interface WeChatPaymentConfig {
  app_id: string
  mch_id: string
  merchant_serial_no: string
  wechatpay_public_key_id: string
  merchant_private_key_pem: string
  api_v3_key: string
  wechatpay_public_key_pem: string
  api_base_url: string
}

export interface AlipayPaymentConfig {
  app_id: string
  app_private_key_pem: string
  alipay_public_key_pem: string
  gateway_url: string
  seller_id: string
}

export type PaymentChannelConfig = Partial<WeChatPaymentConfig & AlipayPaymentConfig>

export interface PaymentChannel {
  id: number
  name: string
  provider: PaymentProvider
  pay_type: string
  status: PaymentChannelStatus
  sort_order: number
  config: PaymentChannelConfig
  created_at: string
  updated_at: string
}

export type ListPaymentChannelsParams = {
  provider?: PaymentProvider
  status?: PaymentChannelStatus
} & ListParamsReq

export interface SavePaymentChannelReq {
  name: string
  provider: PaymentProvider
  pay_type: string
  status: PaymentChannelStatus
  sort_order?: number | null
  config: PaymentChannelConfig
}
