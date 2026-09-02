<template>
  <div class="space-y-5">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h1 class="text-2xl font-semibold text-slate-950">{{ $t('dashboard.title') }}</h1>
        <p class="mt-1 text-sm text-slate-500">{{ $t('dashboard.subtitle') }}</p>
      </div>
      <el-button :loading="loading || trendLoading" @click="reload">{{ $t('common.refresh') }}</el-button>
    </div>

    <div v-loading="loading" class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-4">
      <div v-for="item in summaryCards" :key="item.label" class="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
        <div class="flex items-start justify-between gap-3">
          <div>
            <p class="text-sm text-slate-500">{{ item.label }}</p>
            <p class="mt-2 text-3xl font-semibold text-slate-950">{{ item.value }}</p>
          </div>
          <el-icon class="rounded-lg bg-slate-100 p-2 text-xl text-slate-600">
            <component :is="item.icon" />
          </el-icon>
        </div>
        <p class="mt-3 text-xs text-slate-500">{{ item.note }}</p>
      </div>
    </div>

    <el-card shadow="never">
      <template #header>
        <div class="flex flex-wrap items-start justify-between gap-4">
          <div>
            <div class="font-medium text-slate-950">{{ $t('dashboard.trend_title') }}</div>
            <div class="mt-1 text-xs text-slate-500">{{ trendRangeLabel }}</div>
          </div>
          <div class="flex flex-wrap items-center gap-3">
            <el-select v-model="selectedAppId" class="w-56" :placeholder="$t('dashboard.all_apps')">
              <el-option :label="$t('dashboard.all_apps')" :value="0" />
              <el-option
                v-for="app in trend.apps"
                :key="app.id"
                :label="app.name"
                :value="app.id"
              />
            </el-select>
            <el-radio-group v-model="groupBy" size="small">
              <el-radio-button value="hour">{{ $t('dashboard.by_hour') }}</el-radio-button>
              <el-radio-button value="day">{{ $t('dashboard.by_day') }}</el-radio-button>
              <el-radio-button value="month">{{ $t('dashboard.by_month') }}</el-radio-button>
              <el-radio-button value="year">{{ $t('dashboard.by_year') }}</el-radio-button>
            </el-radio-group>
            <el-date-picker
              v-model="selectedRange"
              :type="trendRangePickerType"
              :format="trendRangePickerFormat"
              :clearable="false"
              unlink-panels
              range-separator="-"
              :start-placeholder="$t('dashboard.start_date')"
              :end-placeholder="$t('dashboard.end_date')"
              class="max-w-full"
              @change="onTrendRangeChange"
            />
            <el-radio-group v-model="valueMode" size="small">
              <el-radio-button value="period">{{ $t('dashboard.period_values') }}</el-radio-button>
              <el-radio-button value="cumulative">{{ $t('dashboard.cumulative_values') }}</el-radio-button>
            </el-radio-group>
            <el-radio-group v-model="chartType" size="small">
              <el-radio-button value="bar">{{ $t('dashboard.bar_chart') }}</el-radio-button>
              <el-radio-button value="line">{{ $t('dashboard.line_chart') }}</el-radio-button>
            </el-radio-group>
          </div>
        </div>
      </template>
      <div v-loading="trendLoading" class="trend-chart-shell">
        <div v-show="hasTrendData" ref="trendChartElement" class="trend-chart-canvas"></div>
        <el-empty
          v-if="!trendLoading && !hasTrendData"
          class="trend-chart-empty"
          :description="$t('dashboard.no_trend_data')"
          :image-size="90"
        />
      </div>
    </el-card>

    <div class="grid grid-cols-1 gap-4 xl:grid-cols-[360px_minmax(0,1fr)]">
      <el-card shadow="never">
        <template #header>
          <div class="flex items-center justify-between">
            <span class="font-medium">{{ $t('dashboard.order_status') }}</span>
            <span class="text-xs text-slate-500">{{ $t('dashboard.total_orders') }} {{ stats.total_orders }}</span>
          </div>
        </template>
        <div class="space-y-4">
          <div v-for="item in statusRows" :key="item.label" class="space-y-2">
            <div class="flex items-center justify-between text-sm">
              <span class="text-slate-600">{{ item.label }}</span>
              <span class="font-medium text-slate-950">{{ item.value }}</span>
            </div>
            <el-progress :percentage="item.percent" :status="item.status" :show-text="false" />
          </div>
        </div>
      </el-card>

      <el-card shadow="never">
        <template #header>
          <div class="flex items-center justify-between">
            <span class="font-medium">{{ $t('dashboard.recent_orders') }}</span>
            <el-button text type="primary" @click="goOrders">{{ $t('dashboard.view_all') }}</el-button>
          </div>
        </template>
        <el-table :data="stats.recent_orders" size="large" stripe style="width: 100%">
          <el-table-column prop="order_no" :label="$t('orders.order_id')" min-width="190" />
          <el-table-column :label="$t('products.name')" min-width="160">
            <template #default="{ row }">{{ row.plan_name || '-' }}</template>
          </el-table-column>
          <el-table-column :label="$t('reg_codes.app')" min-width="140">
            <template #default="{ row }">{{ row.app_name || '-' }}</template>
          </el-table-column>
          <el-table-column :label="$t('orders.final_price')" width="120">
            <template #default="{ row }">&yen;{{ formatPrice(row.amount_cents) }}</template>
          </el-table-column>
          <el-table-column :label="$t('orders.status')" width="110">
            <template #default="{ row }">
              <el-tag :type="statusType(row.status)">{{ orderStatusLabel(row.status) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column :label="$t('orders.created')" min-width="170">
            <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
          </el-table-column>
          <template #empty>
            <el-empty :description="$t('dashboard.no_orders')" :image-size="80" />
          </template>
        </el-table>
      </el-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { BarChart, type BarSeriesOption, LineChart, type LineSeriesOption } from 'echarts/charts'
import {
  DataZoomComponent,
  GridComponent,
  LegendComponent,
  TooltipComponent,
  type DataZoomComponentOption,
  type GridComponentOption,
  type LegendComponentOption,
  type TooltipComponentOption,
} from 'echarts/components'
import { init, use, type ComposeOption, type EChartsType } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { fetchDashboardStats, fetchDashboardTrend } from '@/apis/dashboard'
import type {
  DashboardStats,
  DashboardTrend,
  DashboardTrendGroupBy,
} from '@/types'
import { OrderStatus } from '@/types/payments'
import { RoutePath } from '@/types/route'
import { formatTime } from '@/utils'

type TrendValueMode = 'period' | 'cumulative'
type TrendChartType = 'bar' | 'line'
type TrendChartOption = ComposeOption<
  BarSeriesOption
  | LineSeriesOption
  | DataZoomComponentOption
  | GridComponentOption
  | LegendComponentOption
  | TooltipComponentOption
>

use([
  BarChart,
  LineChart,
  DataZoomComponent,
  GridComponent,
  LegendComponent,
  TooltipComponent,
  CanvasRenderer,
])

const { t, locale } = useI18n()
const router = useRouter()
const loading = ref(false)
const trendLoading = ref(false)
const selectedAppId = ref(0)
const groupBy = ref<DashboardTrendGroupBy>('day')
const selectedRange = ref<[Date, Date]>(defaultTrendRange('day'))
const valueMode = ref<TrendValueMode>('period')
const chartType = ref<TrendChartType>('bar')
const trendChartElement = ref<HTMLElement>()
const trend = reactive<DashboardTrend>({ points: [], apps: [] })
let trendChart: EChartsType | undefined
let resizeObserver: ResizeObserver | undefined
let trendRequestId = 0
const stats = reactive<DashboardStats>({
  total_revenue_cents: 0,
  total_orders: 0,
  total_users: 0,
  new_orders_today: 0,
  pending_orders: 0,
  delivered_orders: 0,
  failed_orders: 0,
  active_products: 0,
  recent_orders: [],
})

const trendRangeLabel = computed(() => {
  const [start, end] = selectedRange.value
  const options: Intl.DateTimeFormatOptions = groupBy.value === 'hour' || groupBy.value === 'day'
    ? { year: 'numeric', month: '2-digit', day: '2-digit' }
    : groupBy.value === 'month'
      ? { year: 'numeric', month: '2-digit' }
      : { year: 'numeric' }
  const formatter = new Intl.DateTimeFormat(locale.value, options)
  return t('dashboard.range_note', {
    start: formatter.format(start),
    end: formatter.format(end),
  })
})

const trendRangePickerType = computed(() => {
  if (groupBy.value === 'hour' || groupBy.value === 'day') return 'daterange'
  if (groupBy.value === 'month') return 'monthrange'
  return 'yearrange'
})

const trendRangePickerFormat = computed(() => {
  if (groupBy.value === 'hour' || groupBy.value === 'day') return 'YYYY-MM-DD'
  if (groupBy.value === 'month') return 'YYYY-MM'
  return 'YYYY'
})

const hasTrendData = computed(() => trend.points.some(
  point => point.revenue_cents !== 0 || point.order_count !== 0,
))

const chartPoints = computed(() => {
  let revenueCents = 0
  let orderCount = 0
  return trend.points.map((point) => {
    if (valueMode.value === 'cumulative') {
      revenueCents += point.revenue_cents
      orderCount += point.order_count
    } else {
      revenueCents = point.revenue_cents
      orderCount = point.order_count
    }
    return {
      period: point.period,
      revenue: revenueCents / 100,
      orders: orderCount,
    }
  })
})

const summaryCards = computed(() => [
  {
    label: t('dashboard.total_revenue'),
    value: `¥${formatPrice(stats.total_revenue_cents)}`,
    note: t('dashboard.revenue_note'),
    icon: 'Money',
  },
  {
    label: t('dashboard.total_orders'),
    value: String(stats.total_orders),
    note: t('dashboard.orders_note', { count: stats.new_orders_today }),
    icon: 'ShoppingCart',
  },
  {
    label: t('dashboard.total_users'),
    value: String(stats.total_users),
    note: t('dashboard.users_note'),
    icon: 'User',
  },
  {
    label: t('dashboard.new_orders'),
    value: String(stats.new_orders_today),
    note: t('dashboard.new_orders_note'),
    icon: 'ShoppingCart',
  },
])

const statusRows = computed(() => [
  {
    label: t('orders.status_0'),
    value: stats.pending_orders,
    percent: percentage(stats.pending_orders),
    status: undefined,
  },
  {
    label: t('orders.status_2'),
    value: stats.delivered_orders,
    percent: percentage(stats.delivered_orders),
    status: 'success' as const,
  },
  {
    label: t('orders.status_3'),
    value: stats.failed_orders,
    percent: percentage(stats.failed_orders),
    status: 'exception' as const,
  },
])

function percentage(value: number) {
  if (!stats.total_orders) return 0
  return Math.round((value / stats.total_orders) * 100)
}

function formatPrice(cents: number) {
  return (cents / 100).toFixed(2)
}

function defaultTrendRange(value: DashboardTrendGroupBy): [Date, Date] {
  const now = new Date()
  if (value === 'hour') {
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
    return [today, today]
  }
  if (value === 'year') {
    return [
      new Date(now.getFullYear() - 4, 0, 1),
      new Date(now.getFullYear(), 0, 1),
    ]
  }
  if (value === 'month') {
    return [
      new Date(now.getFullYear(), now.getMonth() - 11, 1),
      new Date(now.getFullYear(), now.getMonth(), 1),
    ]
  }
  const end = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const start = new Date(end)
  start.setDate(start.getDate() - 29)
  return [start, end]
}

function formatDateForApi(value: Date) {
  const year = value.getFullYear()
  const month = String(value.getMonth() + 1).padStart(2, '0')
  const day = String(value.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

function onTrendRangeChange() {
  if (selectedRange.value.length === 2) void loadTrend()
}

function formatChartCurrency(value: number) {
  return `¥${new Intl.NumberFormat(locale.value, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(value)}`
}

function renderTrendChart() {
  void nextTick(() => {
    if (!trendChartElement.value || !hasTrendData.value) return
    if (!trendChart) trendChart = init(trendChartElement.value)
    const points = chartPoints.value
    const seriesStyle = chartType.value === 'bar'
      ? { barMaxWidth: 34 }
      : { smooth: true, showSymbol: false, symbolSize: 7 }
    const option: TrendChartOption = {
      animationDuration: 300,
      color: ['#2563eb', '#16a34a'],
      tooltip: { trigger: 'axis' },
      legend: { top: 0, left: 0 },
      grid: { top: 52, right: 62, bottom: 68, left: 70, containLabel: false },
      dataZoom: [
        {
          type: 'inside',
          xAxisIndex: 0,
          filterMode: 'none',
          zoomOnMouseWheel: true,
          moveOnMouseWheel: false,
          moveOnMouseMove: true,
          preventDefaultMouseMove: true,
        },
        {
          type: 'slider',
          xAxisIndex: 0,
          filterMode: 'none',
          bottom: 8,
          height: 22,
          showDetail: false,
          borderColor: '#cbd5e1',
          fillerColor: 'rgba(37, 99, 235, 0.14)',
          handleStyle: { color: '#2563eb', borderColor: '#2563eb' },
        },
      ],
      xAxis: {
        type: 'category',
        boundaryGap: chartType.value === 'bar',
        data: points.map(point => point.period),
        axisTick: { alignWithLabel: true },
        axisLabel: {
          hideOverlap: true,
          formatter: (value: string) => groupBy.value === 'hour' || groupBy.value === 'day'
            ? value.slice(5)
            : value,
        },
      },
      yAxis: [
        {
          type: 'value',
          name: t('dashboard.revenue_amount'),
          axisLabel: { formatter: (value: number) => `¥${value.toLocaleString()}` },
          splitLine: { lineStyle: { color: '#e2e8f0' } },
        },
        {
          type: 'value',
          name: t('dashboard.order_count'),
          minInterval: 1,
          axisLabel: { formatter: (value: number) => value.toLocaleString() },
          splitLine: { show: false },
        },
      ],
      series: [
        {
          name: t('dashboard.revenue_amount'),
          type: chartType.value,
          data: points.map(point => point.revenue),
          yAxisIndex: 0,
          tooltip: { valueFormatter: (value: any) => formatChartCurrency(Number(value)) },
          ...seriesStyle,
        } as BarSeriesOption | LineSeriesOption,
        {
          name: t('dashboard.order_count'),
          type: chartType.value,
          data: points.map(point => point.orders),
          yAxisIndex: 1,
          tooltip: { valueFormatter: (value: any) => String(value) },
          ...seriesStyle,
        } as BarSeriesOption | LineSeriesOption,
      ],
    }
    trendChart.setOption(option, true)
  })
}

function orderStatusLabel(status: OrderStatus) {
  return t(`orders.status_${status}`)
}

function statusType(status: OrderStatus) {
  if (status === OrderStatus.Delivered) return 'success'
  if (status === OrderStatus.Pending) return 'warning'
  if (status === OrderStatus.Failed) return 'danger'
  return 'info'
}

function assignStats(data: DashboardStats) {
  stats.total_revenue_cents = data.total_revenue_cents
  stats.total_orders = data.total_orders
  stats.total_users = data.total_users
  stats.new_orders_today = data.new_orders_today
  stats.pending_orders = data.pending_orders
  stats.delivered_orders = data.delivered_orders
  stats.failed_orders = data.failed_orders
  stats.active_products = data.active_products
  stats.recent_orders = data.recent_orders
}

async function loadStats() {
  loading.value = true
  try {
    assignStats(await fetchDashboardStats())
  } finally {
    loading.value = false
  }
}

async function loadTrend() {
  const requestId = ++trendRequestId
  trendLoading.value = true
  try {
    const [start, end] = selectedRange.value
    const data = await fetchDashboardTrend({
      group_by: groupBy.value,
      app_id: selectedAppId.value || undefined,
      start_date: formatDateForApi(start),
      end_date: formatDateForApi(end),
    })
    if (requestId !== trendRequestId) return
    trend.points = data.points
    trend.apps = data.apps
    renderTrendChart()
  } finally {
    if (requestId === trendRequestId) trendLoading.value = false
  }
}

async function reload() {
  await Promise.all([loadStats(), loadTrend()])
}

function goOrders() {
  router.push(RoutePath.AdminOrders)
}

watch(groupBy, (value) => {
  selectedRange.value = defaultTrendRange(value)
  void loadTrend()
})
watch(selectedAppId, () => void loadTrend())
watch([chartType, valueMode, locale], renderTrendChart)

onMounted(async () => {
  await nextTick()
  if (trendChartElement.value) {
    resizeObserver = new ResizeObserver(() => trendChart?.resize())
    resizeObserver.observe(trendChartElement.value)
  }
  await reload()
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  trendChart?.dispose()
})
</script>

<style scoped>
.trend-chart-shell,
.trend-chart-canvas,
.trend-chart-empty {
  min-height: 360px;
}

.trend-chart-canvas {
  width: 100%;
  height: 360px;
}
</style>
