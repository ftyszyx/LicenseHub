<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter, RouterLink } from 'vue-router'
import { ElMessage } from 'element-plus'
import { fetchOrder } from '@/apis/payments'
import type { OrderModel } from '@/types/payments'
import { OrderStatus } from '@/types/payments'
import { RoutePath } from '@/types/route'
import { useAuthStore } from '@/stores/auth'
import { fetchSiteSettings } from '@/apis/system_settings'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const loading = ref(false)
const order = ref<OrderModel | null>(null)
const loadError = ref('')
const registrationEnabled = ref(false)
const guestEmail = ref('')

const orderNo = computed(() => {
  const value = route.query.order_no
  return Array.isArray(value) ? value[0] || '' : value || ''
})

const resultHint = computed(() => {
  const value = route.query.result
  return Array.isArray(value) ? value[0] || '' : value || ''
})

const resultType = computed(() => {
  if (order.value?.status === OrderStatus.Delivered) return 'success'
  if (resultHint.value === 'failed' || order.value?.status === OrderStatus.Failed) return 'error'
  return 'info'
})

const resultTitle = computed(() => {
  if (order.value?.status === OrderStatus.Delivered) return '支付完成'
  if (resultHint.value === 'failed') return '支付结果待确认'
  return '正在确认支付结果'
})

function formatPrice(cents: number) {
  return (cents / 100).toFixed(2)
}

function orderStatusLabel(status: OrderStatus) {
  switch (status) {
    case OrderStatus.Paid:
      return '已支付'
    case OrderStatus.Delivered:
      return '已完成'
    case OrderStatus.Failed:
      return '支付失败'
    case OrderStatus.Closed:
      return '已关闭'
    case OrderStatus.Pending:
    default:
      return '待支付'
  }
}

async function copy(text?: string | null) {
  if (!text) return
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

async function loadOrder() {
  if (!orderNo.value) {
    loadError.value = '缺少订单号，无法查询支付结果'
    return
  }
  loading.value = true
  loadError.value = ''
  try {
    order.value = await fetchOrder(orderNo.value)
    guestEmail.value = sessionStorage.getItem(`licensehub_order_email:${orderNo.value}`) || ''
  } catch {
    loadError.value = '订单查询失败，请稍后刷新页面'
  } finally {
    loading.value = false
  }
}

function registerAndSaveOrder() {
  void router.push({
    path: RoutePath.Register,
    query: {
      redirect: RoutePath.UserOrders,
      ...(guestEmail.value ? { email: guestEmail.value } : {}),
    },
  })
}

onMounted(async () => {
  await loadOrder()
  try {
    registrationEnabled.value = Boolean((await fetchSiteSettings()).registration_enabled)
  } catch {
    registrationEnabled.value = false
  }
})
</script>

<template>
  <section class="min-h-screen bg-slate-50 px-4 py-10 sm:px-6">
    <div class="mx-auto max-w-3xl rounded-lg border border-slate-200 bg-white p-6 shadow-sm">
      <el-skeleton v-if="loading" :rows="7" animated />

      <el-result v-else-if="loadError" icon="warning" title="无法确认支付结果" :sub-title="loadError">
        <template #extra>
          <el-button type="primary" @click="loadOrder">重新查询</el-button>
          <RouterLink :to="RoutePath.Home">
            <el-button>返回首页</el-button>
          </RouterLink>
        </template>
      </el-result>

      <template v-else-if="order">
        <el-result :icon="resultType" :title="resultTitle">
          <template #sub-title>
            <span v-if="order.status === OrderStatus.Delivered">注册码已经生成，请妥善保存。</span>
            <span v-else>如果你已经完成付款，系统可能还在等待支付平台通知。</span>
          </template>
        </el-result>

        <el-descriptions :column="1" border>
          <el-descriptions-item label="订单号">
            <div class="flex flex-wrap items-center gap-2">
              <span class="font-mono text-sm">{{ order.order_no }}</span>
              <el-button size="small" @click="copy(order.order_no)">复制</el-button>
            </div>
          </el-descriptions-item>
          <el-descriptions-item label="商品">{{ order.plan_name }}</el-descriptions-item>
          <el-descriptions-item label="应用">{{ order.app_name }}</el-descriptions-item>
          <el-descriptions-item label="金额">&yen;{{ formatPrice(order.amount_cents) }}</el-descriptions-item>
          <el-descriptions-item label="状态">{{ orderStatusLabel(order.status) }}</el-descriptions-item>
          <el-descriptions-item v-if="order.reg_code" label="注册码">
            <div class="flex flex-wrap items-center gap-2">
              <span class="font-mono text-sm font-semibold">{{ order.reg_code }}</span>
              <el-button size="small" type="primary" @click="copy(order.reg_code)">复制注册码</el-button>
            </div>
          </el-descriptions-item>
        </el-descriptions>

        <div class="mt-6 flex flex-wrap gap-3">
          <el-button type="primary" @click="loadOrder">刷新状态</el-button>
          <el-button
            v-if="order.status === OrderStatus.Delivered && !authStore.isAuthenticated && registrationEnabled"
            type="success"
            plain
            @click="registerAndSaveOrder"
          >
            注册并保存本订单
          </el-button>
          <RouterLink :to="RoutePath.Home">
            <el-button>继续购买</el-button>
          </RouterLink>
        </div>
      </template>
    </div>
  </section>
</template>
