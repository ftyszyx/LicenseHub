<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between gap-4">
      <div>
        <h2 class="text-xl font-semibold text-slate-950">推广中心</h2>
        <div class="mt-1 text-sm text-slate-500">佣金提现由管理员线下转账处理</div>
      </div>
      <el-button type="primary" :icon="Wallet" :disabled="summary.available_amount_cents <= 0 || Boolean(pendingSettlement)" @click="openWithdrawal">
        {{ pendingSettlement ? '提现处理中' : '申请提现' }}
      </el-button>
    </div>

    <section v-loading="loading" class="distribution-summary-grid overflow-hidden rounded-lg border border-slate-200 bg-white">
      <div class="summary-item"><div class="summary-label">待确认佣金</div><div class="summary-value">¥{{ money(summary.pending_amount_cents) }}</div></div>
      <div class="summary-item"><div class="summary-label">可提现佣金</div><div class="summary-value text-emerald-700">¥{{ money(summary.available_amount_cents) }}</div></div>
      <div class="summary-item"><div class="summary-label">提现处理中</div><div class="summary-value">¥{{ money(summary.locked_amount_cents) }}</div></div>
      <div class="summary-item"><div class="summary-label">累计已结算</div><div class="summary-value">¥{{ money(summary.settled_amount_cents) }}</div></div>
    </section>

    <el-alert
      v-if="summary.adjustment_debt_cents > 0"
      type="warning"
      :closable="false"
      :title="`退款佣金待抵扣 ¥${money(summary.adjustment_debt_cents)}，后续可结算佣金会优先抵扣`"
    />

    <section class="rounded-lg border border-slate-200 bg-white p-4">
      <el-form label-width="90px">
        <el-form-item label="推广链接">
          <el-input :model-value="promotionLink" readonly>
            <template #append><el-button :icon="CopyDocument" @click="copyLink">复制</el-button></template>
          </el-input>
        </el-form-item>
        <div class="grid grid-cols-1 gap-x-8 md:grid-cols-3">
          <el-form-item label="佣金比例">{{ (summary.commission_rate_bps / 100).toFixed(2) }}%</el-form-item>
          <el-form-item label="推广订单">{{ summary.order_count }}</el-form-item>
          <el-form-item label="推广销售额">¥{{ money(summary.sales_amount_cents) }}</el-form-item>
        </div>
      </el-form>
    </section>

    <section class="overflow-hidden rounded-lg border border-slate-200 bg-white">
      <el-tabs v-model="activeTab" class="distribution-tabs">
        <el-tab-pane label="佣金明细" name="commissions">
          <el-table :data="commissions">
            <el-table-column prop="order_no" label="订单号" min-width="190" />
            <el-table-column label="订单金额" width="120"><template #default="{ row }">¥{{ money(row.order_amount_cents) }}</template></el-table-column>
            <el-table-column label="佣金" width="110"><template #default="{ row }">¥{{ money(row.commission_amount_cents) }}</template></el-table-column>
            <el-table-column label="可提现" width="110"><template #default="{ row }">¥{{ money(row.available_amount_cents) }}</template></el-table-column>
            <el-table-column label="状态" width="110"><template #default="{ row }"><el-tag :type="commissionStatusType(row.status)">{{ commissionStatusLabel(row.status) }}</el-tag></template></el-table-column>
            <el-table-column label="可结算时间" min-width="180"><template #default="{ row }">{{ formatTime(row.available_at) }}</template></el-table-column>
            <el-table-column label="创建时间" min-width="180"><template #default="{ row }">{{ formatTime(row.created_at) }}</template></el-table-column>
          </el-table>
        </el-tab-pane>

        <el-tab-pane label="提现记录" name="settlements">
          <el-table :data="settlements">
            <el-table-column prop="settlement_no" label="提现单号" min-width="210" />
            <el-table-column label="金额" width="120"><template #default="{ row }">¥{{ money(row.amount_cents) }}</template></el-table-column>
            <el-table-column label="支付宝账户" min-width="180"><template #default="{ row }">{{ row.settlement_account.account }}</template></el-table-column>
            <el-table-column label="状态" width="110"><template #default="{ row }"><el-tag :type="settlementStatusType(row.status)">{{ settlementStatusLabel(row.status) }}</el-tag></template></el-table-column>
            <el-table-column prop="payment_reference" label="打款流水" min-width="170" />
            <el-table-column prop="reject_reason" label="拒绝原因" min-width="180" />
            <el-table-column label="申请时间" min-width="180"><template #default="{ row }">{{ formatTime(row.requested_at) }}</template></el-table-column>
            <el-table-column label="操作" width="100" fixed="right" align="right">
              <template #default="{ row }">
                <el-button v-if="row.status === SettlementStatus.Paid" text type="primary" :icon="View" @click="openProof(row.id, true)">凭证</el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>

        <el-tab-pane label="退款抵扣" name="adjustments">
          <el-table :data="adjustments" empty-text="暂无退款抵扣记录">
            <el-table-column prop="order_no" label="退款订单" min-width="190" />
            <el-table-column label="冲正金额" width="120"><template #default="{ row }">¥{{ money(-row.amount_cents) }}</template></el-table-column>
            <el-table-column label="已抵扣" width="120"><template #default="{ row }">¥{{ money(row.offset_amount_cents) }}</template></el-table-column>
            <el-table-column label="待抵扣" width="120"><template #default="{ row }">¥{{ money(row.remaining_amount_cents) }}</template></el-table-column>
            <el-table-column label="状态" width="110"><template #default="{ row }"><el-tag :type="row.remaining_amount_cents ? 'warning' : 'success'">{{ row.remaining_amount_cents ? '待抵扣' : '已抵扣' }}</el-tag></template></el-table-column>
            <el-table-column label="创建时间" min-width="180"><template #default="{ row }">{{ formatTime(row.created_at) }}</template></el-table-column>
          </el-table>
        </el-tab-pane>
      </el-tabs>
    </section>

    <el-dialog v-model="withdrawalVisible" title="申请提现" width="500px">
      <el-alert type="info" :closable="false" :title="`默认提现全部可用佣金，最低提现 ¥${money(summary.min_withdraw_cents)}`" class="mb-4" />
      <el-form label-width="110px">
        <el-form-item label="提现金额" required>
          <el-input-number v-model="withdrawal.amountYuan" :min="0" :max="summary.available_amount_cents / 100" :precision="2" :step="1" controls-position="right" class="w-full" />
        </el-form-item>
        <el-form-item label="支付宝账号" required><el-input v-model="withdrawal.alipayAccount" maxlength="255" /></el-form-item>
        <el-form-item label="收款人姓名" required><el-input v-model="withdrawal.realName" maxlength="100" /></el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="withdrawalVisible = false">取消</el-button>
        <el-button type="primary" :loading="submitting" @click="submitWithdrawal">提交申请</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { CopyDocument, View, Wallet } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { useRouter } from 'vue-router'
import {
  fetchMyAdjustments,
  fetchMyCommissions,
  fetchMyDistributionSummary,
  fetchMySettlements,
  fetchSettlementProof,
  requestWithdrawal,
} from '@/apis/distribution'
import { RoutePath } from '@/types'
import {
  CommissionStatus,
  SettlementStatus,
  type AdjustmentInfo,
  type CommissionInfo,
  type DistributionSummary,
  type SettlementInfo,
} from '@/types/distribution'

const router = useRouter()
const loading = ref(false)
const submitting = ref(false)
const activeTab = ref('commissions')
const withdrawalVisible = ref(false)
const commissions = ref<CommissionInfo[]>([])
const settlements = ref<SettlementInfo[]>([])
const adjustments = ref<AdjustmentInfo[]>([])
const summary = reactive<DistributionSummary>({
  referral_code: '', commission_rate_bps: 0, pending_amount_cents: 0,
  available_amount_cents: 0, locked_amount_cents: 0, settled_amount_cents: 0,
  adjustment_debt_cents: 0, min_withdraw_cents: 5000, settlement_account: null,
  order_count: 0, sales_amount_cents: 0,
})
const withdrawal = reactive({ amountYuan: 0, alipayAccount: '', realName: '' })
const promotionLink = computed(() => `${window.location.origin}/?ref=${summary.referral_code}`)
const pendingSettlement = computed(() => settlements.value.find(item => item.status === SettlementStatus.Pending))
const money = (cents: number) => (cents / 100).toFixed(2)

function formatTime(value?: string | null) {
  return value ? new Intl.DateTimeFormat('zh-CN', { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(value)) : '-'
}
function commissionStatusLabel(status: CommissionStatus) {
  return ['待确认', '可提现', '提现中', '已结算', '已取消', '已抵扣'][status] || '未知'
}
function commissionStatusType(status: CommissionStatus) {
  if (status === CommissionStatus.Available || status === CommissionStatus.Settled) return 'success'
  if (status === CommissionStatus.Cancelled) return 'danger'
  if (status === CommissionStatus.Offset) return 'info'
  return 'warning'
}
function settlementStatusLabel(status: SettlementStatus) { return ['待处理', '已打款', '已拒绝'][status] || '未知' }
function settlementStatusType(status: SettlementStatus) { return status === SettlementStatus.Paid ? 'success' : status === SettlementStatus.Rejected ? 'danger' : 'warning' }

async function load() {
  loading.value = true
  try {
    const [summaryData, commissionData, settlementData, adjustmentData] = await Promise.all([
      fetchMyDistributionSummary(),
      fetchMyCommissions({ page: 1, page_size: 100 }),
      fetchMySettlements({ page: 1, page_size: 100 }),
      fetchMyAdjustments({ page: 1, page_size: 100 }),
    ])
    Object.assign(summary, summaryData)
    commissions.value = commissionData.list
    settlements.value = settlementData.list
    adjustments.value = adjustmentData.list
  } catch {
    await router.replace(RoutePath.UserHome)
  } finally { loading.value = false }
}

async function copyLink() {
  await navigator.clipboard.writeText(promotionLink.value)
  ElMessage.success('推广链接已复制')
}

function openWithdrawal() {
  if (pendingSettlement.value) return
  withdrawal.amountYuan = summary.available_amount_cents / 100
  withdrawal.alipayAccount = summary.settlement_account?.account || ''
  withdrawal.realName = summary.settlement_account?.real_name || ''
  withdrawalVisible.value = true
}

async function submitWithdrawal() {
  const amountCents = Math.round(withdrawal.amountYuan * 100)
  if (amountCents < summary.min_withdraw_cents || amountCents > summary.available_amount_cents) {
    ElMessage.error(`提现金额应在 ¥${money(summary.min_withdraw_cents)} 至 ¥${money(summary.available_amount_cents)} 之间`)
    return
  }
  if (!withdrawal.alipayAccount.trim() || !withdrawal.realName.trim()) {
    ElMessage.error('请填写支付宝账号和收款人姓名')
    return
  }
  submitting.value = true
  try {
    await requestWithdrawal({
      amount_cents: amountCents,
      alipay_account: withdrawal.alipayAccount.trim(),
      real_name: withdrawal.realName.trim(),
    })
    ElMessage.success('提现申请已提交')
    withdrawalVisible.value = false
    activeTab.value = 'settlements'
    await load()
  } finally { submitting.value = false }
}

async function openProof(id: number, mine: boolean) {
  const blob = await fetchSettlementProof(id, mine)
  const url = URL.createObjectURL(blob)
  window.open(url, '_blank', 'noopener,noreferrer')
  window.setTimeout(() => URL.revokeObjectURL(url), 60_000)
}

onMounted(load)
</script>

<style scoped>
.distribution-summary-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); }
.summary-item { min-width: 0; padding: 16px 18px; border-right: 1px solid #e2e8f0; }
.summary-item:last-child { border-right: 0; }
.summary-label { font-size: 13px; color: #64748b; }
.summary-value { margin-top: 6px; font-size: 22px; font-weight: 600; line-height: 1.25; }
.distribution-tabs :deep(.el-tabs__header) { margin: 0; padding: 0 16px; }
.distribution-tabs :deep(.el-tabs__content) { padding: 0; }
@media (max-width: 900px) { .distribution-summary-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } .summary-item:nth-child(2) { border-right: 0; } .summary-item:nth-child(-n+2) { border-bottom: 1px solid #e2e8f0; } }
</style>
