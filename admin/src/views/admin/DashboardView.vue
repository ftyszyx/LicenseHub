<template>
  <div class="space-y-5">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h1 class="text-2xl font-semibold text-slate-950">{{ $t('dashboard.title') }}</h1>
        <p class="mt-1 text-sm text-slate-500">{{ $t('dashboard.subtitle') }}</p>
      </div>
      <el-button :loading="loading" @click="reload">{{ $t('common.refresh') }}</el-button>
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
import { computed, onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { fetchDashboardStats } from '@/apis/dashboard'
import type { DashboardStats } from '@/types'
import { OrderStatus } from '@/types/payments'
import { RoutePath } from '@/types/route'
import { formatTime } from '@/utils'

const { t } = useI18n()
const router = useRouter()
const loading = ref(false)
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

async function reload() {
  loading.value = true
  try {
    assignStats(await fetchDashboardStats())
  } finally {
    loading.value = false
  }
}

function goOrders() {
  router.push(RoutePath.AdminOrders)
}

onMounted(reload)
</script>
