<template>
  <div class="admin-list-page">
    <div class="admin-list-fixed flex items-center justify-between gap-4">
      <div>
        <h2 class="text-xl font-semibold text-slate-950">佣金与提现</h2>
        <div class="mt-1 text-sm text-slate-500">审核用户提现申请并登记线下转账凭证</div>
      </div>
      <el-button :icon="Refresh" @click="loadActive">刷新</el-button>
    </div>

    <section class="admin-list-panel rounded-lg border border-slate-200 bg-white">
      <el-tabs v-model="activeTab" class="admin-list-tabs distribution-tabs" @tab-change="loadActive">
        <el-tab-pane label="佣金明细" name="commissions">
          <el-table class="admin-list-table" v-loading="loading" :data="commissions" height="100%">
            <el-table-column prop="id" label="ID" width="90" />
            <el-table-column prop="username" label="推广用户" min-width="140" />
            <el-table-column prop="order_no" label="订单号" min-width="190" />
            <el-table-column label="订单金额" width="110"><template #default="{ row }">¥{{ money(row.order_amount_cents) }}</template></el-table-column>
            <el-table-column label="总佣金" width="110"><template #default="{ row }">¥{{ money(row.commission_amount_cents) }}</template></el-table-column>
            <el-table-column label="可提现" width="110"><template #default="{ row }">¥{{ money(row.available_amount_cents) }}</template></el-table-column>
            <el-table-column label="已结算" width="110"><template #default="{ row }">¥{{ money(row.settled_amount_cents) }}</template></el-table-column>
            <el-table-column label="状态" width="110"><template #default="{ row }"><el-tag :type="commissionStatusType(row.status)">{{ commissionStatusLabel(row.status) }}</el-tag></template></el-table-column>
            <el-table-column label="可结算时间" min-width="180"><template #default="{ row }">{{ formatTime(row.available_at) }}</template></el-table-column>
          </el-table>
        </el-tab-pane>

        <el-tab-pane label="提现申请" name="settlements">
          <div class="admin-list-fixed flex items-center gap-3 border-b border-slate-100 p-3">
            <el-select v-model="settlementStatus" clearable placeholder="全部状态" class="w-36" @change="loadSettlements">
              <el-option label="待处理" :value="SettlementStatus.Pending" />
              <el-option label="已打款" :value="SettlementStatus.Paid" />
              <el-option label="已拒绝" :value="SettlementStatus.Rejected" />
            </el-select>
          </div>
          <el-table class="admin-list-table" v-loading="loading" :data="settlements" height="100%">
            <el-table-column prop="settlement_no" label="提现单号" min-width="210" />
            <el-table-column prop="username" label="用户" min-width="120" />
            <el-table-column label="金额" width="120"><template #default="{ row }"><strong>¥{{ money(row.amount_cents) }}</strong></template></el-table-column>
            <el-table-column label="支付宝账户" min-width="190"><template #default="{ row }"><div>{{ row.settlement_account.account }}</div><div class="text-xs text-slate-500">{{ row.settlement_account.real_name }}</div></template></el-table-column>
            <el-table-column label="状态" width="110"><template #default="{ row }"><el-tag :type="settlementStatusType(row.status)">{{ settlementStatusLabel(row.status) }}</el-tag></template></el-table-column>
            <el-table-column prop="payment_reference" label="打款流水" min-width="170" />
            <el-table-column prop="reject_reason" label="拒绝原因" min-width="180" show-overflow-tooltip />
            <el-table-column label="申请时间" min-width="180"><template #default="{ row }">{{ formatTime(row.requested_at) }}</template></el-table-column>
            <el-table-column label="操作" width="210" fixed="right" align="right">
              <template #default="{ row }">
                <template v-if="row.status === SettlementStatus.Pending">
                  <el-button size="small" type="success" :icon="Check" @click="openPayment(row)">确认打款</el-button>
                  <el-button size="small" type="danger" plain :icon="Close" @click="openReject(row)">拒绝</el-button>
                </template>
                <el-button v-else-if="row.status === SettlementStatus.Paid" size="small" text type="primary" :icon="View" @click="openProof(row.id, false)">查看凭证</el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>

        <el-tab-pane label="退款冲正" name="adjustments">
          <el-table class="admin-list-table" v-loading="loading" :data="adjustments" empty-text="暂无退款冲正记录" height="100%">
            <el-table-column prop="id" label="ID" width="90" />
            <el-table-column prop="username" label="用户" min-width="130" />
            <el-table-column prop="order_no" label="退款订单" min-width="190" />
            <el-table-column label="冲正金额" width="120"><template #default="{ row }">¥{{ money(-row.amount_cents) }}</template></el-table-column>
            <el-table-column label="已抵扣" width="120"><template #default="{ row }">¥{{ money(row.offset_amount_cents) }}</template></el-table-column>
            <el-table-column label="待抵扣" width="120"><template #default="{ row }">¥{{ money(row.remaining_amount_cents) }}</template></el-table-column>
            <el-table-column label="状态" width="110"><template #default="{ row }"><el-tag :type="row.remaining_amount_cents ? 'warning' : 'success'">{{ row.remaining_amount_cents ? '待抵扣' : '已抵扣' }}</el-tag></template></el-table-column>
            <el-table-column label="创建时间" min-width="180"><template #default="{ row }">{{ formatTime(row.created_at) }}</template></el-table-column>
          </el-table>
        </el-tab-pane>
      </el-tabs>

      <div class="admin-list-footer flex justify-end border-t border-slate-100 p-3">
        <el-pagination background layout="total, prev, pager, next" :current-page="page" :page-size="pageSize" :total="total" @current-change="changePage" />
      </div>
    </section>

    <el-dialog v-model="paymentDialog.visible" title="确认线下打款" width="520px">
      <el-descriptions :column="1" border class="mb-4">
        <el-descriptions-item label="提现单号">{{ selectedSettlement?.settlement_no }}</el-descriptions-item>
        <el-descriptions-item label="打款金额">¥{{ money(selectedSettlement?.amount_cents || 0) }}</el-descriptions-item>
        <el-descriptions-item label="支付宝账户">{{ selectedSettlement?.settlement_account.account }}（{{ selectedSettlement?.settlement_account.real_name }}）</el-descriptions-item>
      </el-descriptions>
      <el-form label-width="100px">
        <el-form-item label="打款流水" required><el-input v-model="paymentDialog.reference" maxlength="255" /></el-form-item>
        <el-form-item label="打款凭证" required>
          <el-upload
            :auto-upload="false"
            :limit="1"
            accept="image/jpeg,image/png,image/webp,application/pdf"
            :on-change="handleProofChange"
            :on-remove="clearProof"
          >
            <el-button :icon="Upload">选择图片或 PDF</el-button>
            <template #tip><div class="el-upload__tip">JPG、PNG、WebP 或 PDF，最大 5 MB</div></template>
          </el-upload>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="paymentDialog.visible = false">取消</el-button>
        <el-button type="success" :loading="submitting" @click="submitPayment">确认已打款</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="rejectDialog.visible" title="拒绝提现申请" width="480px">
      <el-form label-width="90px"><el-form-item label="拒绝原因" required><el-input v-model="rejectDialog.reason" type="textarea" :rows="4" maxlength="1000" show-word-limit /></el-form-item></el-form>
      <template #footer>
        <el-button @click="rejectDialog.visible = false">取消</el-button>
        <el-button type="danger" :loading="submitting" @click="submitReject">确认拒绝</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { Check, Close, Refresh, Upload, View } from '@element-plus/icons-vue'
import { ElMessage, type UploadFile } from 'element-plus'
import {
  fetchAdminAdjustments,
  fetchAdminCommissions,
  fetchAdminSettlements,
  fetchSettlementProof,
  markSettlementPaid,
  rejectSettlement,
} from '@/apis/distribution'
import {
  CommissionStatus,
  SettlementStatus,
  type AdjustmentInfo,
  type CommissionInfo,
  type SettlementInfo,
} from '@/types/distribution'

const activeTab = ref('commissions')
const loading = ref(false)
const submitting = ref(false)
const commissions = ref<CommissionInfo[]>([])
const settlements = ref<SettlementInfo[]>([])
const adjustments = ref<AdjustmentInfo[]>([])
const selectedSettlement = ref<SettlementInfo>()
const selectedProof = ref<File>()
const settlementStatus = ref<number>()
const page = ref(1)
const pageSize = 20
const total = ref(0)
const paymentDialog = reactive({ visible: false, reference: '' })
const rejectDialog = reactive({ visible: false, reason: '' })
const money = (cents: number) => (cents / 100).toFixed(2)

function formatTime(value?: string | null) { return value ? new Intl.DateTimeFormat('zh-CN', { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(value)) : '-' }
function commissionStatusLabel(status: CommissionStatus) { return ['待确认', '可提现', '提现中', '已结算', '已取消', '已抵扣'][status] || '未知' }
function commissionStatusType(status: CommissionStatus) { return status === CommissionStatus.Available || status === CommissionStatus.Settled ? 'success' : status === CommissionStatus.Cancelled ? 'danger' : status === CommissionStatus.Offset ? 'info' : 'warning' }
function settlementStatusLabel(status: SettlementStatus) { return ['待处理', '已打款', '已拒绝'][status] || '未知' }
function settlementStatusType(status: SettlementStatus) { return status === SettlementStatus.Paid ? 'success' : status === SettlementStatus.Rejected ? 'danger' : 'warning' }

async function loadCommissions() {
  const data = await fetchAdminCommissions({ page: page.value, page_size: pageSize })
  commissions.value = data.list; total.value = data.total
}
async function loadSettlements() {
  page.value = 1
  const data = await fetchAdminSettlements({ page: page.value, page_size: pageSize, status: settlementStatus.value })
  settlements.value = data.list; total.value = data.total
}
async function loadAdjustments() {
  const data = await fetchAdminAdjustments({ page: page.value, page_size: pageSize })
  adjustments.value = data.list; total.value = data.total
}
async function loadActive() {
  loading.value = true
  try {
    if (activeTab.value === 'settlements') await loadSettlements()
    else if (activeTab.value === 'adjustments') await loadAdjustments()
    else await loadCommissions()
  } finally { loading.value = false }
}
function changePage(value: number) { page.value = value; loadActive() }
function openPayment(row: SettlementInfo) { selectedSettlement.value = row; paymentDialog.reference = ''; selectedProof.value = undefined; paymentDialog.visible = true }
function openReject(row: SettlementInfo) { selectedSettlement.value = row; rejectDialog.reason = ''; rejectDialog.visible = true }
function handleProofChange(file: UploadFile) { selectedProof.value = file.raw }
function clearProof() { selectedProof.value = undefined }

async function submitPayment() {
  if (!selectedSettlement.value || !paymentDialog.reference.trim() || !selectedProof.value) { ElMessage.error('请填写打款流水并上传凭证'); return }
  if (selectedProof.value.size > 5 * 1024 * 1024) { ElMessage.error('凭证不能超过 5 MB'); return }
  submitting.value = true
  try {
    await markSettlementPaid(selectedSettlement.value.id, paymentDialog.reference.trim(), selectedProof.value)
    ElMessage.success('打款记录已保存')
    paymentDialog.visible = false
    await loadActive()
  } finally { submitting.value = false }
}
async function submitReject() {
  if (!selectedSettlement.value || !rejectDialog.reason.trim()) { ElMessage.error('请填写拒绝原因'); return }
  submitting.value = true
  try {
    await rejectSettlement(selectedSettlement.value.id, rejectDialog.reason.trim())
    ElMessage.success('提现申请已拒绝，佣金已经释放')
    rejectDialog.visible = false
    await loadActive()
  } finally { submitting.value = false }
}
async function openProof(id: number, mine: boolean) {
  const blob = await fetchSettlementProof(id, mine)
  const url = URL.createObjectURL(blob)
  window.open(url, '_blank', 'noopener,noreferrer')
  window.setTimeout(() => URL.revokeObjectURL(url), 60_000)
}

onMounted(loadActive)
</script>

<style scoped>
.distribution-tabs :deep(.el-tabs__header) { margin: 0; padding: 0 16px; }
.distribution-tabs :deep(.el-tabs__content) { padding: 0; }
</style>
