<script setup lang="ts">
import { computed, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { CopyDocument, Search } from '@element-plus/icons-vue'
import { fetchPublicOrders } from '@/apis/payments'
import type { PublicOrderLookupModel, PublicOrderLookupType } from '@/types/payments'
import { OrderStatus } from '@/types/payments'
import { useI18n } from 'vue-i18n'
import { formatTime } from '@/utils'

const { t, te } = useI18n()
const lookupType = ref<PublicOrderLookupType>('order_no')
const queryValue = ref('')
const loading = ref(false)
const orders = ref<PublicOrderLookupModel[]>([])
const searched = ref(false)

const canSearch = computed(() => queryValue.value.trim().length > 0)
const inputPlaceholder = computed(() => t(`order_query.placeholder_${lookupType.value}`))

function formatPrice(cents: number) {
  return (cents / 100).toFixed(2)
}

function statusType(status: OrderStatus) {
  if (status === OrderStatus.Delivered) return 'success'
  if (status === OrderStatus.Pending) return 'warning'
  if (status === OrderStatus.Failed) return 'danger'
  return 'info'
}

function orderStatusLabel(status: OrderStatus) {
  return t(`orders.status_${status}`)
}

function providerLabel(provider: string) {
  const key = `payment_settings.provider_${provider}`
  return te(key) ? t(key) : provider
}

function resetSearch() {
  queryValue.value = ''
  orders.value = []
  searched.value = false
}

async function searchOrders() {
  const value = queryValue.value.trim()
  if (!value) return
  loading.value = true
  searched.value = true
  orders.value = []
  try {
    orders.value = await fetchPublicOrders({ type: lookupType.value, value })
  } catch {
    orders.value = []
  } finally {
    loading.value = false
  }
}

async function copy(text?: string | null) {
  if (!text) return
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success(t('common.copied'))
  } catch {
    ElMessage.error(t('common.copy_failed'))
  }
}
</script>

<template>
  <section class="min-h-screen bg-slate-50 px-4 py-10 sm:px-6">
    <div class="mx-auto max-w-4xl">
      <div class="mb-6">
        <h1 class="text-2xl font-semibold text-slate-950">{{ $t('order_query.title') }}</h1>
        <p class="mt-2 text-sm text-slate-600">{{ $t('order_query.subtitle') }}</p>
      </div>

      <div class="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
        <div class="flex flex-col gap-4">
          <el-radio-group v-model="lookupType" class="lookup-types" size="large" @change="resetSearch">
            <el-radio-button value="order_no">{{ $t('order_query.lookup_order_no') }}</el-radio-button>
            <el-radio-button value="buyer_email">{{ $t('order_query.lookup_buyer_email') }}</el-radio-button>
            <el-radio-button value="reg_code">{{ $t('order_query.lookup_reg_code') }}</el-radio-button>
          </el-radio-group>

          <div class="flex flex-col gap-3 sm:flex-row">
            <el-input
              v-model="queryValue"
              size="large"
              clearable
              :type="lookupType === 'buyer_email' ? 'email' : 'text'"
              :placeholder="inputPlaceholder"
              @keyup.enter="searchOrders"
            />
            <el-button
              type="primary"
              size="large"
              class="sm:w-32"
              :icon="Search"
              :disabled="!canSearch"
              :loading="loading"
              @click="searchOrders"
            >
              {{ $t('order_query.search') }}
            </el-button>
          </div>
        </div>
      </div>

      <el-skeleton v-if="loading" class="mt-6 rounded-lg border border-slate-200 bg-white p-6" :rows="9" animated />

      <div v-else-if="orders.length" class="mt-6 space-y-4">
        <p class="text-sm text-slate-600">{{ $t('order_query.result_count', { count: orders.length }) }}</p>

        <article
          v-for="order in orders"
          :key="order.id"
          class="rounded-lg border border-slate-200 bg-white p-5 shadow-sm sm:p-6"
        >
          <div class="mb-5 flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
            <div class="min-w-0">
              <div class="text-sm font-medium text-slate-500">{{ $t('orders.order_id') }}</div>
              <div class="mt-1 flex min-w-0 items-center gap-2">
                <span class="break-all font-mono text-base font-semibold text-slate-950">{{ order.order_no }}</span>
                <el-tooltip :content="$t('common.copy')">
                  <el-button
                    :icon="CopyDocument"
                    circle
                    size="small"
                    :aria-label="$t('common.copy')"
                    @click="copy(order.order_no)"
                  />
                </el-tooltip>
              </div>
            </div>
            <el-tag class="shrink-0 self-start" :type="statusType(order.status)" size="large">
              {{ orderStatusLabel(order.status) }}
            </el-tag>
          </div>

          <el-descriptions class="order-details" :column="1" border>
            <el-descriptions-item :label="$t('order_query.provider_trade_no')">
              <span class="break-all font-mono">{{ order.provider_trade_no || '-' }}</span>
            </el-descriptions-item>
            <el-descriptions-item :label="$t('order_query.buyer_email')">
              <span class="break-all">{{ order.buyer_email || '-' }}</span>
            </el-descriptions-item>
            <el-descriptions-item :label="$t('order_query.paid_at')">
              {{ order.paid_at ? formatTime(order.paid_at) : '-' }}
            </el-descriptions-item>
            <el-descriptions-item :label="$t('order_query.amount')">
              &yen;{{ formatPrice(order.amount_cents) }}
            </el-descriptions-item>
            <el-descriptions-item :label="$t('order_query.payment_provider')">
              {{ providerLabel(order.provider) }}
            </el-descriptions-item>
            <el-descriptions-item :label="$t('orders.pay_method')">{{ order.pay_type }}</el-descriptions-item>
            <el-descriptions-item :label="$t('products.name')">{{ order.plan_name || '-' }}</el-descriptions-item>
            <el-descriptions-item :label="$t('reg_codes.app')">{{ order.app_name || '-' }}</el-descriptions-item>
            <el-descriptions-item :label="$t('orders.created')">{{ formatTime(order.created_at) }}</el-descriptions-item>
            <el-descriptions-item v-if="lookupType !== 'buyer_email'" :label="$t('reg_codes.code')">
              <div v-if="order.reg_code" class="flex min-w-0 items-center gap-2">
                <span class="break-all font-mono text-sm font-semibold">{{ order.reg_code }}</span>
                <el-tooltip :content="$t('order_query.copy_reg_code')">
                  <el-button
                    :icon="CopyDocument"
                    circle
                    size="small"
                    type="primary"
                    :aria-label="$t('order_query.copy_reg_code')"
                    @click="copy(order.reg_code)"
                  />
                </el-tooltip>
              </div>
              <span v-else class="text-slate-500">
                {{ order.order_no.startsWith('TEST-')
                  ? $t('order_query.test_order_without_reg_code')
                  : $t('order_query.reg_code_not_generated') }}
              </span>
            </el-descriptions-item>
          </el-descriptions>
        </article>
      </div>

      <div v-else-if="searched" class="mt-6 rounded-lg border border-slate-200 bg-white py-12">
        <el-empty :description="$t('order_query.not_found')" />
      </div>
    </div>
  </section>
</template>

<style scoped>
.lookup-types {
  display: flex;
  width: 100%;
}

.lookup-types :deep(.el-radio-button) {
  flex: 1;
}

.lookup-types :deep(.el-radio-button__inner) {
  display: flex;
  min-height: 48px;
  align-items: center;
  justify-content: center;
  width: 100%;
  padding-inline: 10px;
  line-height: 18px;
  white-space: normal;
}

.order-details :deep(.el-descriptions__label) {
  width: 112px;
  overflow-wrap: anywhere;
}

.order-details :deep(.el-descriptions__content) {
  min-width: 0;
  overflow-wrap: anywhere;
}

@media (min-width: 640px) {
  .lookup-types {
    width: 430px;
  }

  .lookup-types :deep(.el-radio-button__inner) {
    min-height: 40px;
    white-space: nowrap;
  }

  .order-details :deep(.el-descriptions__label) {
    width: 180px;
  }
}
</style>
