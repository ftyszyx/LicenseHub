<template>
  <div class="space-y-4">
    <el-card shadow="hover">
      <div class="flex items-center justify-between">
        <h2 class="text-xl font-semibold">{{ $t('reg_codes.title') }}</h2>
        <div class="flex items-center gap-2">
          <el-input v-model="query.code" :placeholder="$t('reg_codes.search_code')" clearable class="w-56" />
          <el-select v-model.number="query.app_id" placeholder="App" clearable class="w-48">
            <el-option v-for="opt in appOptions" :key="opt.id" :label="opt.name" :value="opt.id" />
          </el-select>
          <el-select v-model="query.code_type" placeholder="Type" clearable class="w-36" @change="reload">
            <el-option :label="$t('reg_codes.type_time')" :value="RegCodeType.Time" />
            <el-option :label="$t('reg_codes.type_count')" :value="RegCodeType.Count" />
          </el-select>
          <el-select v-model="query.status" :placeholder="$t('reg_codes.status')" clearable class="w-36">
            <el-option v-for="opt in statusOptions" :key="opt.value" :label="opt.label" :value="opt.value" />
          </el-select>
          <el-button type="primary" @click="reload">{{ $t('common.search') }}</el-button>
          <el-button @click="resetFilters">{{ $t('common.reset') }}</el-button>
          <el-button type="success" @click="openBatchDialog">{{ $t('reg_codes.batch_create') }}</el-button>
          <el-button type="danger" :disabled="!selectedIds.length" @click="batchDelete">{{ $t('reg_codes.batch_delete')
          }}</el-button>
          <el-button @click="exportCsv">{{ $t('reg_codes.export_csv') }}</el-button>
        </div>
      </div>
    </el-card>

    <el-card shadow="never">
      <el-table :data="rows" stripe size="large" style="width: 100%" @selection-change="onSelChange">
        <el-table-column type="selection" width="50" />
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column :label="$t('reg_codes.code')" min-width="200">
          <template #default="{ row }">
            <div class="flex items-center gap-2">
              <span class="text-gray-800 break-all">{{ row.code }}</span>
              <el-button size="small" @click="copy(row.code)">{{ $t('common.copy') }}</el-button>
            </div>
          </template>
        </el-table-column>
        <el-table-column :label="$t('reg_codes.app')" min-width="160">
          <template #default="{ row }">
            <span>{{ row.app_name || row.app_id }}</span>
          </template>
        </el-table-column>
        <template v-if="query.code_type === RegCodeType.Time">
          <el-table-column prop="valid_days" :label="$t('reg_codes.valid_days')" width="120">
          </el-table-column>
          <el-table-column prop="binding_time" :label="$t('reg_codes.binding_time')" width="180">
            <template #default="{ row }">{{ formatTime(row.binding_time) }}</template>
          </el-table-column>
        </template>
        <template v-if="query.code_type === RegCodeType.Count">
          <el-table-column prop="total_count" :label="$t('reg_codes.total_count')" width="120">
          </el-table-column>
        </template>
        <el-table-column prop="status" :label="$t('reg_codes.status')" width="100">
          <template #default="{ row }">
            <el-tag :type="statusTagType(row.status)" effect="light">{{ $t(`reg_codes.status_${RegCodeStatus[row.status].toLowerCase()}`) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="device_id" :label="$t('reg_codes.device_id')" min-width="180" />
        <el-table-column :label="$t('reg_codes.created')" min-width="180">
          <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
        </el-table-column>
        <el-table-column :label="$t('common.actions')" width="220" fixed="right" align="right">
          <template #default="{ row }">
            <div class="flex justify-end gap-2">
              <el-button v-if="row.status === RegCodeStatus.Unused" size="small" type="primary" plain @click="markIssued(row)">{{ $t('reg_codes.mark_issued') }}</el-button>
              <el-button v-if="row.status === RegCodeStatus.binded" size="small" type="warning" plain @click="revoke(row)">{{ $t('reg_codes.revoke') }}</el-button>
              <el-button v-if="row.status === RegCodeStatus.Unused" size="small" type="danger" plain @click="del(row.id)">{{ $t('common.delete') }}</el-button>
            </div>
          </template>
        </el-table-column>
      </el-table>
      <div class="flex justify-end mt-4">
        <el-pagination background layout="total, sizes, prev, pager, next, jumper" :page-sizes="[10, 20, 50, 100]"
          :page-size="pageSize" :current-page="page" :total="total" @current-change="handlePageChange"
          @size-change="handleSizeChange" />
      </div>
    </el-card>

    <el-dialog v-model="batch.visible" :title="$t('reg_codes.batch_create')" width="520px">
      <el-form label-width="140px">
        <el-form-item :label="$t('reg_codes.app')">
          <el-select v-model.number="batch.app_id" placeholder="Select App" class="w-full">
            <el-option v-for="opt in appOptions" :key="opt.id" :label="opt.name" :value="opt.id" />
          </el-select>
        </el-form-item>
        <el-form-item :label="$t('reg_codes.quantity')"><el-input-number v-model.number="batch.quantity"
            :min="1" /></el-form-item>
        <el-form-item :label="$t('reg_codes.type')">
          <el-tag effect="light">{{ batchCodeTypeLabel }}</el-tag>
        </el-form-item>
        <el-form-item v-if="batch.code_type === RegCodeType.Time" :label="$t('reg_codes.valid_days')"><el-input-number
            v-model.number="batch.valid_days" :min="1" /></el-form-item>
        <el-form-item v-else :label="$t('reg_codes.total_count')"><el-input-number v-model.number="batch.total_count"
            :min="1" /></el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="batch.visible = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submitBatch">{{ $t('common.confirm') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, reactive, onMounted, watch } from 'vue'
import { fetchRegCodes, deleteRegCode, batchCreateRegCodes, updateRegCodeStatus, revokeRegCode } from '@/apis/reg_codes'
import { fetchApps } from '@/apis/apps'
import type { RegCodeModel, ListRegCodesParams, BatchCreateRegCodesReq } from '@/types/reg_codes'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useI18n } from 'vue-i18n'
import { RegCodeStatus, RegCodeType } from '@/types/reg_codes'
import { formatTime } from '@/utils'
const { t } = useI18n()

const statusOptions = (Object.values(RegCodeStatus).filter(v => typeof v === 'number') as number[]).map(v => {
  const status = v as RegCodeStatus
  return {
    value: status,
    label: t(`reg_codes.status_${RegCodeStatus[status].toLowerCase()}`),
  }
})

function statusTagType(status: RegCodeStatus) {
  if (status === RegCodeStatus.Unused) return 'info'
  if (status === RegCodeStatus.Issued) return 'success'
  if (status === RegCodeStatus.binded) return 'warning'
  if (status === RegCodeStatus.Refunded) return 'danger'
  return 'info'
}

const rows = ref<RegCodeModel[]>([])
const appOptions = ref<{ id: number, name: string, code_type: RegCodeType }[]>([])
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const selectedIds = ref<number[]>([])

const query = reactive<ListRegCodesParams>({ code: '', app_id: undefined, code_type: RegCodeType.Time })

async function reload() {
  const data = await fetchRegCodes({ ...query, page: page.value, page_size: pageSize.value })
  rows.value = data.list
  total.value = data.total
}
function resetFilters() { query.code = ''; query.app_id = undefined; query.code_type = RegCodeType.Time; query.status = undefined; page.value = 1; reload() }
function onSelChange(arr: RegCodeModel[]) { selectedIds.value = arr.map(it => it.id) }
function handlePageChange(p: number) { page.value = p; reload() }
function handleSizeChange(s: number) { pageSize.value = s; page.value = 1; reload() }

async function del(id: number) {
  await ElMessageBox.confirm(t('reg_codes.delete_code_confirm'), t('common.confirm'), { type: 'warning' })
  await deleteRegCode(id)
  ElMessage.success(t('common.deleted'))
  reload()
}

async function markIssued(row: RegCodeModel) {
  if (row.status === RegCodeStatus.binded) {
    ElMessage.warning(t('reg_codes.status_locked'))
    return
  }
  await ElMessageBox.confirm(t('reg_codes.mark_issued_confirm'), t('common.confirm'), { type: 'warning' })
  await updateRegCodeStatus(row.id, RegCodeStatus.Issued)
  ElMessage.success(t('common.saved'))
  reload()
}
async function revoke(row: RegCodeModel) {
  await ElMessageBox.confirm(t('reg_codes.revoke_confirm'), t('common.confirm'), { type: 'warning' })
  await revokeRegCode(row.id)
  ElMessage.success(t('reg_codes.revoked'))
  reload()
}
async function batchDelete() {
  const hasLocked = rows.value.some(r => selectedIds.value.includes(r.id) && r.status !== RegCodeStatus.Unused)
  if (hasLocked) {
    ElMessage.warning(t('reg_codes.delete_locked'))
    return
  }
  await ElMessageBox.confirm(t('reg_codes.delete_selected_codes_confirm'), t('common.confirm'), { type: 'warning' })
  for (const id of selectedIds.value) { await deleteRegCode(id) }
  ElMessage.success(t('common.deleted'))
  reload()
}
function exportCsv() {
  const headers = ['id', 'code', 'app_id', 'app_name', 'code_type', 'valid_days', 'total_count', 'status', 'created_at']
  const lines = rows.value.map(r => [
    r.id, r.code, r.app_id, (r.app_name || ''), r.code_type, r.valid_days, (r.total_count ?? ''), r.status, r.created_at
  ].join(','))
  const csv = [headers.join(','), ...lines].join('\n')
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a'); a.href = url; a.download = 'reg_codes.csv'; a.click(); URL.revokeObjectURL(url)
}

const batch = reactive<BatchCreateRegCodesReq & { visible: boolean }>({ visible: false, app_id: 0, quantity: 10, code_type: 0, valid_days: 7, total_count: 1 })
const selectedBatchApp = computed(() => appOptions.value.find(app => app.id === batch.app_id) || null)
const batchCodeTypeLabel = computed(() => batch.code_type === RegCodeType.Time ? t('reg_codes.type_time') : t('reg_codes.type_count'))

function applyBatchAppType() {
  const app = selectedBatchApp.value
  if (!app) return
  batch.code_type = app.code_type
  if (app.code_type === RegCodeType.Time) {
    batch.valid_days = batch.valid_days && batch.valid_days > 0 ? batch.valid_days : 7
    batch.total_count = null
    return
  }
  batch.valid_days = null
  batch.total_count = batch.total_count && batch.total_count > 0 ? batch.total_count : 1
}
function openBatchDialog() {
  if ((!batch.app_id || batch.app_id === 0) && appOptions.value.length) {
    batch.app_id = appOptions.value[0].id
  }
  applyBatchAppType()
  batch.visible = true
}
async function submitBatch() {
  applyBatchAppType()
  await batchCreateRegCodes(batch)
  batch.visible = false
  ElMessage.success(t('common.created'))
  reload()
}

async function copy(text: string) { try { await navigator.clipboard.writeText(text); ElMessage.success(t('common.copied')) } catch { ElMessage.error(t('common.copy_failed')) } }

onMounted(async () => { await loadApps(); await reload() })

async function loadApps() {
  const data = await fetchApps({ page: 1, page_size: 1000 })
  appOptions.value = data.list.map((a: any) => ({ id: a.id, name: a.name, code_type: a.code_type as RegCodeType }))
  if ((!batch.app_id || batch.app_id === 0) && appOptions.value.length) { batch.app_id = appOptions.value[0].id }
  applyBatchAppType()
}

watch(() => batch.app_id, applyBatchAppType)
</script>

<style scoped></style>
