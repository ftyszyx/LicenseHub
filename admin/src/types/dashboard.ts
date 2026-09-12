import type { OrderStatus } from './payments'
import type { ListParamsReq } from './api'

export interface DashboardRecentOrder {
  id: number
  order_no: string
  plan_name?: string | null
  app_name?: string | null
  amount_cents: number
  status: OrderStatus
  created_at: string
}

export interface DashboardStats {
  total_revenue_cents: number
  total_orders: number
  total_users: number
  new_orders_today: number
  pending_orders: number
  delivered_orders: number
  failed_orders: number
  active_products: number
  recent_orders: DashboardRecentOrder[]
}

export type DashboardTrendGroupBy = 'hour' | 'day' | 'month' | 'year'

export interface DashboardTrendParams {
  group_by: DashboardTrendGroupBy
  app_id?: number
  start_date?: string
  end_date?: string
}

export interface DashboardTrendPoint {
  period: string
  revenue_cents: number
  order_count: number
  new_buyer_order_count: number
  returning_buyer_order_count: number
  unidentified_buyer_order_count: number
}

export interface DashboardTrendApp {
  id: number
  name: string
}

export interface DashboardTrend {
  points: DashboardTrendPoint[]
  apps: DashboardTrendApp[]
}

export interface DashboardReturningOrdersParams extends ListParamsReq {
  group_by: DashboardTrendGroupBy
  period: string
  app_id?: number
}

export interface DashboardReturningOrder {
  order_no: string
  provider_trade_no?: string | null
  first_order_no: string
  purchase_number: number
  app_id: number
  app_name: string
  amount_cents: number
  provider: string
  pay_type: string
  provider_buyer_id: string
  buyer_user_id?: number | null
  buyer_username?: string | null
  buyer_user_email?: string | null
  buyer_email?: string | null
  paid_at?: string | null
  created_at: string
}
