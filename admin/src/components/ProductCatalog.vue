<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { createOrder, fetchOrder, fetchPayMethods, fetchPublicPlans } from '@/apis/payments'
import type { LicensePlan, OrderModel, PayMethodInfo, PayMethodsInfo } from '@/types/payments'
import { OrderStatus } from '@/types/payments'
import { RegCodeType } from '@/types/reg_codes'

const props = withDefaults(defineProps<{
  appId?: number | null
  grouped?: boolean
  title?: string
}>(), {
  appId: null,
  grouped: false,
  title: '',
})

type AppPlanGroup = {
  appId: number
  appName: string
  plans: LicensePlan[]
}

const plans = ref<LicensePlan[]>([])
const loading = ref(false)
const creatingId = ref<number | null>(null)
const selectedPlan = ref<LicensePlan | null>(null)
const checkoutDialogVisible = ref(false)
const activeOrder = ref<OrderModel | null>(null)
const payType = ref('')
const payMethodsInfo = ref<PayMethodsInfo | null>(null)
const payMethodsLoading = ref(false)
const activeAppKey = ref('')
let pollTimer: number | undefined

const appGroups = computed<AppPlanGroup[]>(() => {
  const groups = new Map<number, AppPlanGroup>()
  for (const plan of plans.value) {
    const group = groups.get(plan.app_id)
    if (group) {
      group.plans.push(plan)
      continue
    }
    groups.set(plan.app_id, {
      appId: plan.app_id,
      appName: plan.app_name || `App #${plan.app_id}`,
      plans: [plan],
    })
  }
  return Array.from(groups.values())
})

const activeGroup = computed(() => {
  if (!props.grouped) return null
  return appGroups.value.find(group => String(group.appId) === activeAppKey.value) || null
})

const visiblePlans = computed(() => {
  if (!props.grouped) return plans.value
  return activeGroup.value?.plans || []
})

const activePayUrl = computed(() => {
  const order = activeOrder.value
  return order?.pay_url || order?.qr_code || order?.url_scheme || ''
})

const orderDialogVisible = computed({
  get: () => !!activeOrder.value,
  set: (visible: boolean) => {
    if (!visible) {
      activeOrder.value = null
      stopPolling()
    }
  },
})

const enabledPayMethods = computed(() => {
  return payMethodsInfo.value?.methods.filter(method => method.enabled) || []
})

const canConfirmBuy = computed(() => {
  return !!selectedPlan.value && !!payType.value && enabledPayMethods.value.length > 0
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

function planTypeLabel(plan: LicensePlan) {
  return plan.code_type === RegCodeType.Time ? '时间授权' : '次数授权'
}

function planLimit(plan: LicensePlan) {
  if (plan.code_type === RegCodeType.Time) return `${plan.valid_days} 天`
  return `${plan.total_count ?? 0} 次`
}

function planDescription(plan: LicensePlan) {
  return plan.description || '标准授权商品，适合常规激活与续费场景。'
}

async function loadPlans() {
  loading.value = true
  try {
    plans.value = await fetchPublicPlans(props.appId ? { app_id: props.appId } : {})
  } finally {
    loading.value = false
  }
}

async function buy(plan: LicensePlan) {
  selectedPlan.value = plan
  payType.value = ''
  checkoutDialogVisible.value = true
  await loadPayMethods()
}

async function confirmBuy() {
  const plan = selectedPlan.value
  if (!plan || !payType.value) return
  creatingId.value = plan.id
  const payWindow = window.open('', '_blank')
  try {
    activeOrder.value = await createOrder({ plan_id: plan.id, pay_type: payType.value })
    checkoutDialogVisible.value = false
    selectedPlan.value = null
    if (activePayUrl.value) {
      if (payWindow) {
        payWindow.location.href = activePayUrl.value
      } else {
        window.open(activePayUrl.value, '_blank')
      }
    } else if (payWindow) {
      payWindow.close()
    }
    startPolling()
  } catch (error) {
    payWindow?.close()
    throw error
  } finally {
    creatingId.value = null
  }
}

function startPolling() {
  stopPolling()
  if (!activeOrder.value) return
  pollTimer = window.setInterval(async () => {
    if (!activeOrder.value) return
    const latest = await fetchOrder(activeOrder.value.order_no)
    activeOrder.value = latest
    if (latest.status === OrderStatus.Delivered) {
      stopPolling()
      ElMessage.success('支付已完成')
    }
  }, 3000)
}

function stopPolling() {
  if (pollTimer) window.clearInterval(pollTimer)
  pollTimer = undefined
}

function openPayUrl(url: string) {
  window.open(url, '_blank')
}

async function copy(text: string) {
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

async function loadPayMethods() {
  payMethodsLoading.value = true
  try {
    const info = await fetchPayMethods()
    payMethodsInfo.value = info
    payType.value = info.methods.find(method => method.enabled)?.pay_type || ''
  } finally {
    payMethodsLoading.value = false
  }
}

function selectPayMethod(method: PayMethodInfo) {
  if (!method.enabled) return
  payType.value = method.pay_type
}

function payMethodMeta(method: PayMethodInfo) {
  if (method.pay_type === 'alipay') {
    return {
      name: method.label || '支付宝',
      icon: '/static/images/pay/alipay.png',
    }
  }
  if (method.pay_type === 'wxpay') {
    return {
      name: method.label || '微信',
      icon: '/static/images/pay/wxpay.png',
    }
  }
  if (method.pay_type === 'qqpay') {
    return {
      name: method.label || 'QQ 钱包',
      icon: '/static/images/pay/card.svg',
    }
  }
  if (method.pay_type === 'bank') {
    return {
      name: method.label || '银行卡',
      icon: '/static/images/pay/card.svg',
    }
  }
  return {
    name: method.label || '在线支付',
    icon: '/static/images/pay/card.svg',
  }
}

watch(() => props.appId, loadPlans, { immediate: true })

watch(appGroups, (groups) => {
  if (!props.grouped || groups.length === 0) return
  if (!groups.some(group => String(group.appId) === activeAppKey.value)) {
    activeAppKey.value = String(groups[0].appId)
  }
}, { immediate: true })

onBeforeUnmount(stopPolling)
</script>

<template>
  <section class="min-h-screen bg-slate-50">
    <div class="mx-auto max-w-6xl px-4 py-8 sm:px-6">
      <div v-if="loading" class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
        <div v-for="index in 3" :key="index" class="rounded-lg border border-slate-200 bg-white p-5">
          <el-skeleton :rows="5" animated />
        </div>
      </div>
      <template v-else>
        <div v-if="grouped && appGroups.length" class="mb-6">
          <div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
            <div class="flex gap-2 overflow-x-auto rounded-lg border border-slate-200 bg-white p-1">
              <button
                v-for="group in appGroups"
                :key="group.appId"
                type="button"
                class="flex shrink-0 items-center gap-2 rounded-md px-4 py-2 text-sm font-medium transition"
                :class="activeAppKey === String(group.appId) ? 'bg-slate-950 text-white shadow-sm' : 'text-slate-600 hover:bg-slate-100 hover:text-slate-950'"
                @click="activeAppKey = String(group.appId)"
              >
                <span>{{ group.appName }}</span>
                <span
                  class="rounded px-1.5 py-0.5 text-xs"
                  :class="activeAppKey === String(group.appId) ? 'bg-white/15 text-white' : 'bg-slate-100 text-slate-500'"
                >
                  {{ group.plans.length }}
                </span>
              </button>
            </div>
          </div>
        </div>

        <div v-if="visiblePlans.length" class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          <article
            v-for="plan in visiblePlans"
            :key="plan.id"
            class="flex min-h-72 flex-col rounded-lg border border-slate-200 bg-white p-5 transition hover:-translate-y-0.5 hover:border-slate-300 hover:shadow-lg hover:shadow-slate-200/70"
          >
            <div class="flex items-start justify-between gap-4">
              <div>
                <div v-if="!appId" class="text-sm font-medium text-slate-500">{{ plan.app_name || `App #${plan.app_id}` }}</div>
                <h2 class="mt-2 text-xl font-semibold text-slate-950">{{ plan.name }}</h2>
              </div>
              <span class="shrink-0 rounded-md bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-600">
                {{ planTypeLabel(plan) }}
              </span>
            </div>

            <p class="mt-4 min-h-12 text-sm leading-6 text-slate-600">{{ planDescription(plan) }}</p>

            <div class="mt-5 grid grid-cols-2 gap-3 border-y border-slate-100 py-4">
              <div>
                <div class="text-xs font-medium text-slate-500">授权额度</div>
                <div class="mt-1 text-sm font-semibold text-slate-950">{{ planLimit(plan) }}</div>
              </div>
              <div>
                <div class="text-xs font-medium text-slate-500">交付方式</div>
                <div class="mt-1 text-sm font-semibold text-slate-950">自动发码</div>
              </div>
            </div>

            <div class="mt-auto pt-5">
              <div class="mb-4 flex items-end justify-between gap-3">
                <div>
                  <div class="text-xs font-medium text-slate-500">售价</div>
                  <div class="mt-1 text-3xl font-semibold text-slate-950">
                    <span class="text-base">&yen;</span>{{ formatPrice(plan.price_cents) }}
                  </div>
                </div>
              </div>
              <button
                type="button"
                class="flex h-11 w-full items-center justify-center rounded-lg bg-slate-950 px-4 text-sm font-semibold text-white transition hover:bg-slate-800 disabled:cursor-not-allowed disabled:bg-slate-400"
                :disabled="creatingId === plan.id"
                @click="buy(plan)"
              >
                {{ creatingId === plan.id ? '处理中...' : $t('products_page.buy_now') }}
              </button>
            </div>
          </article>
        </div>
        <div v-else class="rounded-lg border border-slate-200 bg-white py-12">
          <el-empty :description="$t('products_page.empty')" />
        </div>
      </template>

      <el-dialog v-model="checkoutDialogVisible" title="确认订单" width="720px">
        <div v-if="selectedPlan" class="space-y-4">
          <div class="rounded-lg border border-slate-200 bg-slate-50 p-4">
            <div class="flex items-start justify-between gap-4">
              <div>
                <div class="text-sm font-medium text-slate-500">{{ selectedPlan.app_name || `App #${selectedPlan.app_id}` }}</div>
                <div class="mt-1 text-lg font-semibold text-slate-950">{{ selectedPlan.name }}</div>
                <div class="mt-2 text-sm text-slate-600">{{ planTypeLabel(selectedPlan) }} · {{ planLimit(selectedPlan) }}</div>
              </div>
              <div class="text-right">
                <div class="text-xs font-medium text-slate-500">应付金额</div>
                <div class="mt-1 text-2xl font-semibold text-slate-950">&yen;{{ formatPrice(selectedPlan.price_cents) }}</div>
              </div>
            </div>
          </div>
          <div class="text-sm font-semibold text-slate-800">选择支付方式</div>
          <el-skeleton v-if="payMethodsLoading" :rows="2" animated />
          <div v-else-if="enabledPayMethods.length" class="flex flex-wrap gap-2">
            <button
              v-for="method in enabledPayMethods"
              :key="method.pay_type"
              type="button"
              class="relative flex h-11 min-w-32 items-center justify-center gap-2 rounded border bg-white px-4 text-sm text-slate-600 transition hover:border-blue-400"
              :class="payType === method.pay_type ? 'border-blue-500 bg-blue-50 text-blue-700' : 'border-slate-200'"
              @click="selectPayMethod(method)"
            >
              <img :src="payMethodMeta(method).icon" :alt="payMethodMeta(method).name" class="h-7 w-7 object-contain">
              <span class="font-medium">{{ payMethodMeta(method).name }}</span>
              <span
                v-if="payType === method.pay_type"
                class="absolute right-0 top-0 flex h-4 w-5 items-center justify-center rounded-bl-sm bg-blue-500 text-[10px] leading-none text-white"
              >
                ✓
              </span>
            </button>
          </div>
          <el-alert
            v-else
            type="warning"
            :closable="false"
            :title="payMethodsInfo?.message || '当前商户没有可用支付方式'"
          />
        </div>
        <template #footer>
          <el-button @click="checkoutDialogVisible = false">取消</el-button>
          <el-button type="primary" class="min-w-32" :disabled="!canConfirmBuy" :loading="creatingId === selectedPlan?.id" @click="confirmBuy">
            确认购买
          </el-button>
        </template>
      </el-dialog>

      <el-dialog v-model="orderDialogVisible" title="订单支付" width="560px">
        <div v-if="activeOrder" class="space-y-4">
          <el-descriptions :column="1" border>
            <el-descriptions-item label="订单号">
              <div class="flex flex-wrap items-center gap-2">
                <span class="font-mono text-sm">{{ activeOrder.order_no }}</span>
                <el-button size="small" @click="copy(activeOrder.order_no)">复制</el-button>
              </div>
            </el-descriptions-item>
            <el-descriptions-item label="商品">{{ activeOrder.plan_name }}</el-descriptions-item>
            <el-descriptions-item label="金额">&yen;{{ formatPrice(activeOrder.amount_cents) }}</el-descriptions-item>
            <el-descriptions-item label="状态">{{ orderStatusLabel(activeOrder.status) }}</el-descriptions-item>
          </el-descriptions>

          <div v-if="activePayUrl && activeOrder.status !== OrderStatus.Delivered" class="rounded-lg border border-blue-100 bg-blue-50 p-4">
            <div class="text-sm font-medium text-slate-900">支付页面已打开</div>
            <p class="mt-1 text-sm text-slate-600">如果浏览器拦截或支付页被关闭，可以重新打开支付页面。</p>
            <el-button class="mt-3" type="primary" @click="openPayUrl(activePayUrl)">重新打开支付页</el-button>
          </div>

          <el-result v-if="activeOrder.status === OrderStatus.Delivered" icon="success" title="支付完成">
            <template #sub-title>
              <div class="mt-2 text-base">
                注册码：
                <span class="font-mono font-semibold">{{ activeOrder.reg_code }}</span>
              </div>
            </template>
            <template #extra>
              <el-button v-if="activeOrder.reg_code" type="primary" @click="copy(activeOrder.reg_code)">复制注册码</el-button>
            </template>
          </el-result>
        </div>
      </el-dialog>
    </div>
  </section>
</template>
