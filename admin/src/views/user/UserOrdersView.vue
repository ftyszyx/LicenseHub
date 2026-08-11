<template>
  <div class="admin-list-page">
    <div class="admin-list-fixed flex flex-wrap items-center justify-between gap-3">
      <div>
        <h2 class="text-xl font-semibold text-slate-950">我的订单</h2>
        <p class="mt-1 text-sm text-slate-500">查看登录账号购买或通过验证邮箱认领的订单与注册码。</p>
      </div>
      <el-button :icon="Refresh" :loading="loading" @click="load">刷新</el-button>
    </div>

    <section class="admin-list-panel rounded-lg border border-slate-200 bg-white p-4">
      <div class="admin-list-fixed mb-4 flex flex-wrap gap-3">
        <el-input v-model="filters.order_no" clearable placeholder="订单号" class="!w-64" @keyup.enter="search" />
        <el-select v-model="filters.status" clearable placeholder="全部状态" class="!w-36" @change="search">
          <el-option v-for="item in statusOptions" :key="item.value" :label="item.label" :value="item.value" />
        </el-select>
        <el-button type="primary" :icon="Search" @click="search">查询</el-button>
      </div>

      <el-table class="admin-list-table" v-loading="loading" :data="orders" row-key="id" empty-text="暂无订单" height="100%">
        <el-table-column prop="order_no" label="订单号" min-width="190">
          <template #default="{ row }"><span class="font-mono text-xs">{{ row.order_no }}</span></template>
        </el-table-column>
        <el-table-column prop="app_name" label="应用" min-width="130" />
        <el-table-column prop="plan_name" label="商品" min-width="150" />
        <el-table-column label="金额" width="100">
          <template #default="{ row }">&yen;{{ (row.amount_cents / 100).toFixed(2) }}</template>
        </el-table-column>
        <el-table-column label="状态" width="105">
          <template #default="{ row }"><el-tag :type="statusMeta(row.status).type">{{ statusMeta(row.status).label }}</el-tag></template>
        </el-table-column>
        <el-table-column label="注册码" min-width="210">
          <template #default="{ row }">
            <div v-if="row.reg_code" class="flex items-center gap-2">
              <span class="truncate font-mono text-xs font-semibold">{{ row.reg_code }}</span>
              <el-button link type="primary" :icon="CopyDocument" @click="copy(row.reg_code)">复制</el-button>
            </div>
            <span v-else class="text-slate-400">待支付完成后生成</span>
          </template>
        </el-table-column>
        <el-table-column label="创建时间" min-width="175">
          <template #default="{ row }">{{ formatDate(row.created_at) }}</template>
        </el-table-column>
      </el-table>

      <div class="admin-list-footer mt-4 flex justify-end">
        <el-pagination
          v-model:current-page="page"
          :page-size="pageSize"
          :total="total"
          layout="total, prev, pager, next"
          @current-change="load"
        />
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { CopyDocument, Refresh, Search } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { fetchMyOrders } from '@/apis/payments'
import { OrderStatus, type OrderModel } from '@/types/payments'

const loading = ref(false)
const orders = ref<OrderModel[]>([])
const page = ref(1)
const pageSize = 20
const total = ref(0)
const filters = reactive<{ order_no: string; status?: OrderStatus }>({ order_no: '', status: undefined })
const statusOptions = [
  { label: '待支付', value: OrderStatus.Pending },
  { label: '已支付', value: OrderStatus.Paid },
  { label: '已完成', value: OrderStatus.Delivered },
  { label: '支付失败', value: OrderStatus.Failed },
  { label: '已关闭', value: OrderStatus.Closed },
  { label: '已退款', value: OrderStatus.Refunded },
]

type TagType = 'success' | 'warning' | 'info'

function statusMeta(status: OrderStatus): { label: string; type: TagType } {
  const option = statusOptions.find(item => item.value === status)
  if (!option) return { label: '未知', type: 'info' }
  const type: TagType = status === OrderStatus.Delivered
    ? 'success'
    : status === OrderStatus.Refunded
      ? 'warning'
      : 'info'
  return { label: option.label, type }
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat('zh-CN', { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(value))
}

async function copy(value?: string | null) {
  if (!value) return
  await navigator.clipboard.writeText(value)
  ElMessage.success('注册码已复制')
}

function search() {
  page.value = 1
  void load()
}

async function load() {
  loading.value = true
  try {
    const result = await fetchMyOrders({
      page: page.value,
      page_size: pageSize,
      order_no: filters.order_no.trim() || undefined,
      status: filters.status,
    })
    orders.value = result.list
    total.value = result.total
  } finally {
    loading.value = false
  }
}

onMounted(load)
</script>
