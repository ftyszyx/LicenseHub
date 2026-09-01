import type { ListParamsReq } from './api'

export type SystemLogLevel = 'INFO' | 'WARN' | 'ERROR'

export interface SystemLogEntry {
  timestamp: string
  level: SystemLogLevel | string
  target: string
  message: string
}

export interface SystemLogPage {
  list: SystemLogEntry[]
  page: number
  total: number
  selected_date: string | null
  available_dates: string[]
}

export type ListSystemLogsParams = {
  date?: string
  level?: SystemLogLevel
  keyword?: string
} & ListParamsReq
