<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { RouterLink } from 'vue-router'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { toDataURL } from 'qrcode'
import { createOrder, fetchOrder, fetchPayMethods, fetchPublicPlans } from '@/apis/payments'
import { fetchSiteSettings } from '@/apis/system_settings'
import type {
  LicensePlan,
  OrderModel,
  PayMethodInfo,
  PayMethodsInfo,
  PublicPlansInfo,
  PublicPlansState,
} from '@/types/payments'
import { OrderStatus } from '@/types/payments'
import { RegCodeType } from '@/types/reg_codes'
import { RoutePath } from '@/types'
import { useAuthStore } from '@/stores/auth'

const props = withDefaults(defineProps<{
  appId?: number | null
  grouped?: boolean
  title?: string
}>(), {
  appId: null,
  grouped: false,
  title: '',
})

const { t } = useI18n()
const router = useRouter()
const authStore = useAuthStore()

type AppPlanGroup = {
  appId: number
  appName: string
  plans: LicensePlan[]
}

const plans = ref<LicensePlan[]>([])
const catalogState = ref<PublicPlansState>('available')
const catalogAppName = ref<string | null>(null)
const loading = ref(false)
const creatingId = ref<number | null>(null)
const selectedPlan = ref<LicensePlan | null>(null)
const checkoutDialogVisible = ref(false)
const activeOrder = ref<OrderModel | null>(null)
const payType = ref('')
const payMethodsInfo = ref<PayMethodsInfo | null>(null)
const payMethodsLoading = ref(false)
const activeAppKey = ref('')
const activeQrCodeDataUrl = ref('')
let pollTimer: number | undefined
const referralCode = ref('')
const registrationEnabled = ref(false)
const distributionEnabled = ref(false)
const guestEmail = ref('')
let checkoutIntentRestored = false

async function initializeReferral() {
  const settings = await fetchSiteSettings()
  registrationEnabled.value = Boolean(settings.registration_enabled)
  distributionEnabled.value = Boolean(settings.distribution?.enabled)
  const storageKey = 'licensehub_referral'
  if (!settings.distribution?.enabled) {
    localStorage.removeItem(storageKey)
    referralCode.value = ''
    return
  }
  const ref = new URLSearchParams(window.location.search).get('ref')?.trim().toUpperCase()
  if (ref) {
    const expiresAt = Date.now() + settings.distribution.attribution_days * 86400000
    localStorage.setItem(storageKey, JSON.stringify({ code: ref, expiresAt }))
    referralCode.value = ref
    return
  }
  try {
    const saved = JSON.parse(localStorage.getItem(storageKey) || 'null')
    if (saved?.code && saved?.expiresAt > Date.now()) referralCode.value = saved.code
    else localStorage.removeItem(storageKey)
  } catch {
    localStorage.removeItem(storageKey)
  }
}

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

const unavailableTitle = computed(() => {
  if (catalogState.value === 'app_disabled') return t('products_page.app_disabled_title')
  if (catalogState.value === 'app_not_found') return t('products_page.app_not_found_title')
  return ''
})

const unavailableDescription = computed(() => {
  if (catalogState.value === 'app_disabled') {
    return t('products_page.app_disabled_description', {
      name: catalogAppName.value || t('products_page.current_app'),
    })
  }
  if (catalogState.value === 'app_not_found') return t('products_page.app_not_found_description')
  return ''
})

const activePayUrl = computed(() => {
  const order = activeOrder.value
  return order?.pay_url || order?.url_scheme || ''
})

const activeQrCode = computed(() => {
  const order = activeOrder.value
  return order?.pay_type === 'wechat_native' ? order?.qr_code || '' : ''
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
  return !!selectedPlan.value
    && !!payType.value
    && enabledPayMethods.value.length > 0
    && (authStore.isAuthenticated || validEmail(guestEmail.value))
})

const registrationPrompt = computed(() => distributionEnabled.value
  ? '注册后可长期保存订单和注册码，还能分享推广链接，好友购买后获得佣金。'
  : '注册后可长期保存订单和注册码，登录后随时查看。')

function validEmail(value: string) {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value.trim())
}

function formatPrice(cents: number) {
  return (cents / 100).toFixed(2)
}

function orderStatusLabel(status: OrderStatus) {
  return t(`orders.status_${status}`)
}

function planTypeLabel(plan: LicensePlan) {
  return plan.code_type === RegCodeType.Time
    ? t('products_page.time_license')
    : t('products_page.count_license')
}

function planLimit(plan: LicensePlan) {
  if (plan.code_type === RegCodeType.Time) {
    return t('products_page.valid_days', { count: plan.valid_days })
  }
  return t('products_page.total_count', { count: plan.total_count ?? 0 })
}

function planDescription(plan: LicensePlan) {
  return plan.description || t('products_page.default_description')
}

async function loadPlans() {
  loading.value = true
  try {
    const result: PublicPlansInfo = await fetchPublicPlans(props.appId ? { app_id: props.appId } : {})
    catalogState.value = result.state
    catalogAppName.value = result.app_name || null
    plans.value = result.plans
    await restoreCheckoutIntent()
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

async function restoreCheckoutIntent() {
  if (checkoutIntentRestored || !authStore.isAuthenticated) return
  checkoutIntentRestored = true
  try {
    const intent = JSON.parse(sessionStorage.getItem('licensehub_checkout_intent') || 'null')
    if (!intent?.planId || intent.expiresAt < Date.now()) {
      sessionStorage.removeItem('licensehub_checkout_intent')
      return
    }
    const plan = plans.value.find(item => item.id === intent.planId)
    if (!plan) return
    sessionStorage.removeItem('licensehub_checkout_intent')
    await buy(plan)
    if (enabledPayMethods.value.some(item => item.pay_type === intent.payType)) {
      payType.value = intent.payType
    }
  } catch {
    sessionStorage.removeItem('licensehub_checkout_intent')
  }
}

function saveCheckoutIntent(plan?: LicensePlan | null) {
  sessionStorage.setItem('licensehub_checkout_intent', JSON.stringify({
    planId: plan?.id || selectedPlan.value?.id || null,
    payType: payType.value || null,
    referralCode: referralCode.value || null,
    expiresAt: Date.now() + 30 * 60 * 1000,
  }))
}

function goRegister(plan?: LicensePlan | null, redirect?: string) {
  saveCheckoutIntent(plan)
  void router.push({
    path: RoutePath.Register,
    query: {
      redirect: redirect || `${window.location.pathname}${window.location.search}`,
      ...(guestEmail.value ? { email: guestEmail.value.trim() } : {}),
    },
  })
}

function goLogin() {
  void router.push(RoutePath.Login)
}

async function confirmBuy() {
  const plan = selectedPlan.value
  if (!plan || !payType.value) return
  creatingId.value = plan.id
  const payWindow = payType.value === 'wechat_native' ? null : window.open('', '_blank')
  try {
    const order = await createOrder({
      plan_id: plan.id,
      pay_type: payType.value,
      referral_code: referralCode.value || undefined,
      buyer_email: authStore.isAuthenticated ? undefined : guestEmail.value.trim(),
    })
    activeOrder.value = order
    if (!authStore.isAuthenticated) {
      sessionStorage.setItem(`licensehub_order_email:${order.order_no}`, guestEmail.value.trim())
    }
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
    const currentOrder = activeOrder.value
    const latest = await fetchOrder(activeOrder.value.order_no)
    activeOrder.value = {
      ...latest,
      pay_url: latest.pay_url || currentOrder.pay_url,
      qr_code: latest.qr_code || currentOrder.qr_code,
      url_scheme: latest.url_scheme || currentOrder.url_scheme,
    }
    if (latest.status === OrderStatus.Delivered) {
      stopPolling()
      ElMessage.success(t('products_page.payment_completed'))
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
    ElMessage.success(t('common.copied'))
  } catch {
    ElMessage.error(t('common.copy_failed'))
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
  if (method.pay_type === 'wechat_native') {
    return {
      name: method.label || t('products_page.pay_wechat_native'),
      icon: '/static/images/pay/wxpay.png',
    }
  }
  if (method.pay_type === 'alipay') {
    return {
      name: method.label || t('products_page.pay_alipay'),
      icon: '/static/images/pay/alipay.png',
    }
  }
  if (method.pay_type === 'wxpay') {
    return {
      name: method.label || t('products_page.pay_wechat'),
      icon: '/static/images/pay/wxpay.png',
    }
  }
  if (method.pay_type === 'qqpay') {
    return {
      name: method.label || t('products_page.pay_qq'),
      icon: '/static/images/pay/card.svg',
    }
  }
  if (method.pay_type === 'bank') {
    return {
      name: method.label || t('products_page.pay_bank'),
      icon: '/static/images/pay/card.svg',
    }
  }
  return {
    name: method.label || t('products_page.pay_online'),
    icon: '/static/images/pay/card.svg',
  }
}

function payMethodsMessage(info: PayMethodsInfo | null) {
  if (info?.message === 'payment is disabled') return t('products_page.payment_disabled')
  if (info?.message === 'no payment channel is configured') return t('products_page.no_payment_channel')
  return info?.message || t('products_page.no_pay_methods')
}

watch(() => props.appId, loadPlans, { immediate: true })
void initializeReferral()

watch(activeQrCode, async (code) => {
  if (!code) {
    activeQrCodeDataUrl.value = ''
    return
  }
  activeQrCodeDataUrl.value = await toDataURL(code, {
    width: 240,
    margin: 1,
    errorCorrectionLevel: 'M',
  })
})

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
        <div
          v-if="!authStore.isAuthenticated && registrationEnabled"
          class="mb-5 flex flex-col gap-3 rounded-lg border border-emerald-200 bg-emerald-50 px-4 py-3 sm:flex-row sm:items-center sm:justify-between"
        >
          <p class="text-sm leading-6 text-emerald-950">{{ registrationPrompt }}</p>
          <div class="flex shrink-0 gap-2">
            <el-button type="success" plain @click="goRegister(null)">立即注册</el-button>
            <el-button @click="goLogin">登录</el-button>
          </div>
        </div>

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

        <div v-if="catalogState !== 'available'" class="rounded-lg border border-slate-200 bg-white px-4 py-12">
          <el-result icon="warning" :title="unavailableTitle" :sub-title="unavailableDescription">
            <template #extra>
              <RouterLink v-slot="{ navigate }" custom :to="RoutePath.Home">
                <el-button type="primary" @click="navigate">{{ $t('products_page.back_home') }}</el-button>
              </RouterLink>
            </template>
          </el-result>
        </div>

        <div v-else-if="visiblePlans.length" class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
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
                <div class="text-xs font-medium text-slate-500">{{ $t('products_page.license_quota') }}</div>
                <div class="mt-1 text-sm font-semibold text-slate-950">{{ planLimit(plan) }}</div>
              </div>
              <div>
                <div class="text-xs font-medium text-slate-500">{{ $t('products_page.delivery_method') }}</div>
                <div class="mt-1 text-sm font-semibold text-slate-950">{{ $t('products_page.auto_delivery') }}</div>
              </div>
            </div>

            <div class="mt-auto pt-5">
              <div class="mb-4 flex items-end justify-between gap-3">
                <div>
                  <div class="text-xs font-medium text-slate-500">{{ $t('products_page.sale_price') }}</div>
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
                {{ creatingId === plan.id ? $t('products_page.processing') : $t('products_page.buy_now') }}
              </button>
            </div>
          </article>
        </div>
        <div v-else class="rounded-lg border border-slate-200 bg-white py-12">
          <el-empty :description="$t('products_page.empty')" />
        </div>
      </template>

      <el-dialog v-model="checkoutDialogVisible" :title="$t('products_page.confirm_order')" width="720px">
        <div v-if="selectedPlan" class="space-y-4">
          <div class="rounded-lg border border-slate-200 bg-slate-50 p-4">
            <div class="flex items-start justify-between gap-4">
              <div>
                <div class="text-sm font-medium text-slate-500">{{ selectedPlan.app_name || `App #${selectedPlan.app_id}` }}</div>
                <div class="mt-1 text-lg font-semibold text-slate-950">{{ selectedPlan.name }}</div>
                <div class="mt-2 text-sm text-slate-600">{{ planTypeLabel(selectedPlan) }} · {{ planLimit(selectedPlan) }}</div>
              </div>
              <div class="text-right">
                <div class="text-xs font-medium text-slate-500">{{ $t('products_page.payable_amount') }}</div>
                <div class="mt-1 text-2xl font-semibold text-slate-950">&yen;{{ formatPrice(selectedPlan.price_cents) }}</div>
              </div>
            </div>
          </div>
          <div v-if="!authStore.isAuthenticated" class="rounded-lg border border-slate-200 p-4">
            <div class="text-sm font-semibold text-slate-800">购买邮箱</div>
            <p class="mt-1 text-xs leading-5 text-slate-500">用于找回订单；注册并验证此邮箱后，订单会自动保存到账号。</p>
            <el-input v-model="guestEmail" class="mt-3" maxlength="320" autocomplete="email" placeholder="name@example.com" />
            <div v-if="registrationEnabled" class="mt-3 flex flex-wrap items-center gap-2 text-xs text-slate-500">
              <span>也可以先注册，购买后直接进入“我的订单”。</span>
              <el-button link type="primary" @click="goRegister(selectedPlan)">注册账号</el-button>
            </div>
          </div>
          <div class="text-sm font-semibold text-slate-800">{{ $t('products_page.select_pay_method') }}</div>
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
            :title="payMethodsMessage(payMethodsInfo)"
          />
        </div>
        <template #footer>
          <el-button @click="checkoutDialogVisible = false">{{ $t('common.cancel') }}</el-button>
          <el-button type="primary" class="min-w-32" :disabled="!canConfirmBuy" :loading="creatingId === selectedPlan?.id" @click="confirmBuy">
            {{ $t('products_page.confirm_buy') }}
          </el-button>
        </template>
      </el-dialog>

      <el-dialog v-model="orderDialogVisible" :title="$t('products_page.order_payment')" width="560px">
        <div v-if="activeOrder" class="space-y-4">
          <el-descriptions :column="1" border>
            <el-descriptions-item :label="$t('orders.order_id')">
              <div class="flex flex-wrap items-center gap-2">
                <span class="font-mono text-sm">{{ activeOrder.order_no }}</span>
                <el-button size="small" @click="copy(activeOrder.order_no)">{{ $t('common.copy') }}</el-button>
              </div>
            </el-descriptions-item>
            <el-descriptions-item :label="$t('products.name')">{{ activeOrder.plan_name }}</el-descriptions-item>
            <el-descriptions-item :label="$t('products_page.amount')">&yen;{{ formatPrice(activeOrder.amount_cents) }}</el-descriptions-item>
            <el-descriptions-item :label="$t('orders.status')">{{ orderStatusLabel(activeOrder.status) }}</el-descriptions-item>
          </el-descriptions>

          <div v-if="activeQrCodeDataUrl && activeOrder.status !== OrderStatus.Delivered" class="rounded-lg border border-emerald-100 bg-emerald-50 p-4">
            <div class="flex flex-col items-center gap-3 text-center">
              <img :src="activeQrCodeDataUrl" :alt="$t('products_page.wechat_qr_alt')" class="h-60 w-60 rounded bg-white p-2 shadow-sm">
              <div>
                <div class="text-sm font-medium text-slate-900">{{ $t('products_page.scan_wechat') }}</div>
                <p class="mt-1 text-sm text-slate-600">{{ $t('products_page.auto_refresh_hint') }}</p>
              </div>
            </div>
          </div>

          <div v-if="activePayUrl && activeOrder.status !== OrderStatus.Delivered" class="rounded-lg border border-blue-100 bg-blue-50 p-4">
            <div class="text-sm font-medium text-slate-900">{{ $t('products_page.pay_page_opened') }}</div>
            <p class="mt-1 text-sm text-slate-600">{{ $t('products_page.reopen_pay_hint') }}</p>
            <el-button class="mt-3" type="primary" @click="openPayUrl(activePayUrl)">{{ $t('products_page.reopen_pay_page') }}</el-button>
          </div>

          <el-result v-if="activeOrder.status === OrderStatus.Delivered" icon="success" :title="$t('products_page.payment_completed')">
            <template #sub-title>
              <div class="mt-2 text-base">
                {{ $t('products_page.reg_code') }}:
                <span class="font-mono font-semibold">{{ activeOrder.reg_code }}</span>
              </div>
            </template>
            <template #extra>
              <el-button v-if="activeOrder.reg_code" type="primary" @click="copy(activeOrder.reg_code)">{{ $t('order_query.copy_reg_code') }}</el-button>
              <el-button
                v-if="!authStore.isAuthenticated && registrationEnabled"
                type="success"
                plain
                @click="goRegister(null, RoutePath.UserOrders)"
              >
                注册并保存本订单
              </el-button>
            </template>
          </el-result>
        </div>
      </el-dialog>
    </div>
  </section>
</template>
