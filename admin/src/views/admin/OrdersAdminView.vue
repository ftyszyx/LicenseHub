<template>
  <div class="space-y-4">
    <el-card shadow="hover">
      <div class="flex items-center justify-between gap-3">
        <h2 class="text-xl font-semibold">{{ $t('orders.title') }}</h2>
        <div class="flex items-center gap-2">
          <el-input v-model="query.order_no" :placeholder="$t('orders.search_order_id')" clearable class="w-56" />
          <el-select v-model="query.status" :placeholder="$t('orders.status')" clearable class="w-36">
            <el-option v-for="opt in statusOptions" :key="opt.value" :label="opt.label" :value="opt.value" />
          </el-select>
          <el-button type="primary" @click="reload">{{ $t('common.search') }}</el-button>
          <el-button @click="reset">{{ $t('common.reset') }}</el-button>
        </div>
      </div>
    </el-card>

    <el-card shadow="never">
      <el-table :data="rows" stripe size="large" style="width: 100%">
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="order_no" :label="$t('orders.order_id')" min-width="210" />
        <el-table-column :label="$t('products.name')" min-width="160">
          <template #default="{ row }">{{ row.plan_name || row.plan_id }}</template>
        </el-table-column>
        <el-table-column :label="$t('reg_codes.app')" min-width="150">
          <template #default="{ row }">{{ row.app_name || row.app_id }}</template>
        </el-table-column>
        <el-table-column :label="$t('orders.final_price')" width="120">
          <template #default="{ row }">¥{{ formatPrice(row.amount_cents) }}</template>
        </el-table-column>
        <el-table-column prop="pay_type" :label="$t('orders.pay_method')" width="110" />
        <el-table-column :label="$t('orders.status')" width="120">
          <template #default="{ row }">
            <el-tag :type="statusType(row.status)">{{ orderStatusLabel(row.status) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="provider_trade_no" label="Trade No" min-width="180" />
        <el-table-column :label="$t('orders.refund_reference')" min-width="180">
          <template #default="{ row }">
            <span v-if="row.refund" class="font-mono text-sm">{{ row.refund.refund_reference }}</span>
            <span v-else>-</span>
          </template>
        </el-table-column>
        <el-table-column :label="$t('reg_codes.code')" min-width="190">
          <template #default="{ row }">
            <div v-if="row.reg_code" class="flex items-center gap-2">
              <span class="break-all font-mono">{{ row.reg_code }}</span>
              <el-button size="small" @click="copy(row.reg_code)">{{ $t('common.copy') }}</el-button>
            </div>
            <span v-else>-</span>
          </template>
        </el-table-column>
        <el-table-column :label="$t('orders.created')" min-width="180">
          <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
        </el-table-column>
        <el-table-column :label="$t('common.actions')" width="120" fixed="right" align="right">
          <template #default="{ row }">
            <el-button
              v-if="row.status === OrderStatus.Delivered"
              size="small"
              type="danger"
              plain
              @click="openRefund(row)"
            >
              {{ $t('orders.confirm_refund') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="mt-4 flex justify-end">
        <el-pagination background layout="total, sizes, prev, pager, next, jumper" :page-sizes="[10, 20, 50, 100]"
          :page-size="pageSize" :current-page="page" :total="total" @current-change="handlePageChange"
          @size-change="handleSizeChange" />
      </div>
    </el-card>

    <el-dialog v-model="refundDialog.visible" :title="$t('orders.confirm_refund')" width="520px">
      <el-alert
        type="warning"
        :closable="false"
        :title="$t('orders.manual_refund_warning')"
        class="mb-4"
      />
      <el-descriptions :column="1" border class="mb-4">
        <el-descriptions-item :label="$t('orders.order_id')">{{ refundOrder?.order_no }}</el-descriptions-item>
        <el-descriptions-item :label="$t('orders.final_price')">¥{{ formatPrice(refundOrder?.amount_cents || 0) }}</el-descriptions-item>
        <el-descriptions-item label="Trade No">{{ refundOrder?.provider_trade_no || '-' }}</el-descriptions-item>
      </el-descriptions>
      <el-form label-width="110px">
        <el-form-item :label="$t('orders.refund_reference')" required>
          <el-input v-model="refundDialog.refund_reference" maxlength="255" />
        </el-form-item>
        <el-form-item :label="$t('orders.refund_reason')" required>
          <el-input v-model="refundDialog.reason" type="textarea" :rows="3" maxlength="1000" show-word-limit />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="refundDialog.visible = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="danger" :loading="refundDialog.submitting" @click="submitRefund">
          {{ $t('orders.refund_recorded_confirm') }}
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { confirmOrderRefund, fetchOrders } from '@/apis/payments'
import type { ListOrdersParams, OrderModel } from '@/types/payments'
import { OrderStatus } from '@/types/payments'
import { formatTime } from '@/utils'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const rows = ref<OrderModel[]>([])
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const query = reactive<ListOrdersParams>({ order_no: '', status: undefined })
const refundOrder = ref<OrderModel>()
const refundDialog = reactive({
  visible: false,
  submitting: false,
  refund_reference: '',
  reason: '',
})

const statusOptions = (Object.values(OrderStatus).filter(v => typeof v === 'number') as number[]).map(value => ({
  value,
  label: t(`orders.status_${value}`),
}))

function formatPrice(cents: number) { return (cents / 100).toFixed(2) }

function orderStatusLabel(status: OrderStatus) {
  return t(`orders.status_${status}`)
}

function statusType(status: OrderStatus) {
  if (status === OrderStatus.Delivered) return 'success'
  if (status === OrderStatus.Refunded) return 'danger'
  if (status === OrderStatus.Pending) return 'warning'
  if (status === OrderStatus.Failed) return 'danger'
  return 'info'
}

function openRefund(row: OrderModel) {
  refundOrder.value = row
  refundDialog.refund_reference = ''
  refundDialog.reason = ''
  refundDialog.visible = true
}

async function submitRefund() {
  if (!refundOrder.value) return
  const refundReference = refundDialog.refund_reference.trim()
  const reason = refundDialog.reason.trim()
  if (!refundReference || !reason) {
    ElMessage.error(String(t('common.please_check_form')))
    return
  }
  refundDialog.submitting = true
  try {
    await confirmOrderRefund(refundOrder.value.id, {
      refund_reference: refundReference,
      reason,
    })
    ElMessage.success(String(t('orders.refund_success')))
    refundDialog.visible = false
    await reload()
  } finally {
    refundDialog.submitting = false
  }
}

async function reload() {
  const data = await fetchOrders({ ...query, page: page.value, page_size: pageSize.value })
  rows.value = data.list
  total.value = data.total
}

function reset() {
  query.order_no = ''
  query.status = undefined
  page.value = 1
  reload()
}

function handlePageChange(p: number) { page.value = p; reload() }
function handleSizeChange(s: number) { pageSize.value = s; page.value = 1; reload() }

async function copy(text: string) {
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('Copied')
  } catch {
    ElMessage.error('Copy failed')
  }
}

onMounted(reload)
</script>
