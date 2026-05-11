<script setup lang="ts">
import { computed, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { Search } from '@element-plus/icons-vue'
import { fetchOrder } from '@/apis/payments'
import type { OrderModel } from '@/types/payments'
import { OrderStatus } from '@/types/payments'
import { useI18n } from 'vue-i18n'
import { formatTime } from '@/utils'

const { t } = useI18n()
const orderNo = ref('')
const loading = ref(false)
const order = ref<OrderModel | null>(null)
const searched = ref(false)

const canSearch = computed(() => orderNo.value.trim().length > 0)

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

async function searchOrder() {
  const value = orderNo.value.trim()
  if (!value) return
  loading.value = true
  searched.value = true
  order.value = null
  try {
    order.value = await fetchOrder(value)
  } catch {
    order.value = null
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
        <div class="flex flex-col gap-3 sm:flex-row">
          <el-input
            v-model="orderNo"
            size="large"
            clearable
            :placeholder="$t('order_query.input_placeholder')"
            @keyup.enter="searchOrder"
          />
          <el-button
            type="primary"
            size="large"
            class="sm:w-32"
            :icon="Search"
            :disabled="!canSearch"
            :loading="loading"
            @click="searchOrder"
          >
            {{ $t('order_query.search') }}
          </el-button>
        </div>
      </div>

      <el-skeleton v-if="loading" class="mt-6 rounded-lg border border-slate-200 bg-white p-6" :rows="7" animated />

      <div v-else-if="order" class="mt-6 rounded-lg border border-slate-200 bg-white p-6 shadow-sm">
        <div class="mb-5 flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <div class="text-sm font-medium text-slate-500">{{ $t('orders.order_id') }}</div>
            <div class="mt-1 flex flex-wrap items-center gap-2">
              <span class="font-mono text-base font-semibold text-slate-950">{{ order.order_no }}</span>
              <el-button size="small" @click="copy(order.order_no)">{{ $t('common.copy') }}</el-button>
            </div>
          </div>
          <el-tag :type="statusType(order.status)" size="large">{{ orderStatusLabel(order.status) }}</el-tag>
        </div>

        <el-descriptions :column="1" border>
          <el-descriptions-item :label="$t('products.name')">{{ order.plan_name }}</el-descriptions-item>
          <el-descriptions-item :label="$t('reg_codes.app')">{{ order.app_name }}</el-descriptions-item>
          <el-descriptions-item :label="$t('orders.final_price')">&yen;{{ formatPrice(order.amount_cents) }}</el-descriptions-item>
          <el-descriptions-item :label="$t('orders.pay_method')">{{ order.pay_type }}</el-descriptions-item>
          <el-descriptions-item :label="$t('orders.created')">{{ formatTime(order.created_at) }}</el-descriptions-item>
          <el-descriptions-item v-if="order.reg_code" :label="$t('reg_codes.code')">
            <div class="flex flex-wrap items-center gap-2">
              <span class="font-mono text-sm font-semibold">{{ order.reg_code }}</span>
              <el-button size="small" type="primary" @click="copy(order.reg_code)">{{ $t('order_query.copy_reg_code') }}</el-button>
            </div>
          </el-descriptions-item>
        </el-descriptions>
      </div>

      <div v-else-if="searched" class="mt-6 rounded-lg border border-slate-200 bg-white py-12">
        <el-empty :description="$t('order_query.not_found')" />
      </div>
    </div>
  </section>
</template>
