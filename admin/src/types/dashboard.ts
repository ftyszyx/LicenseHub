import type { OrderStatus } from './payments'

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
}

export interface DashboardTrendApp {
  id: number
  name: string
}

export interface DashboardTrend {
  points: DashboardTrendPoint[]
  apps: DashboardTrendApp[]
}
