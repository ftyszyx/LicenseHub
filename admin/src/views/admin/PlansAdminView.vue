<template>
  <div class="space-y-4">
    <el-card shadow="hover">
      <div class="flex items-center justify-between gap-3">
        <h2 class="text-xl font-semibold">{{ $t('products.title') }}</h2>
        <div class="flex items-center gap-2">
          <el-input v-model="query.name" :placeholder="$t('products.search_name')" clearable class="w-48" />
          <el-select v-model.number="query.app_id" placeholder="App" clearable class="w-48">
            <el-option v-for="app in appOptions" :key="app.id" :label="app.name" :value="app.id" />
          </el-select>
          <el-select v-model="query.status" :placeholder="$t('products.status')" clearable class="w-32">
            <el-option :label="$t('products.status_1')" :value="PlanStatus.Enabled" />
            <el-option :label="$t('products.status_0')" :value="PlanStatus.Disabled" />
          </el-select>
          <el-button type="primary" @click="reload">{{ $t('common.search') }}</el-button>
          <el-button @click="reset">{{ $t('common.reset') }}</el-button>
          <el-button type="success" @click="openCreate">{{ $t('common.create') }}</el-button>
        </div>
      </div>
    </el-card>

    <el-card shadow="never">
      <el-table :data="rows" stripe size="large" style="width: 100%">
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="name" :label="$t('products.name')" min-width="180" />
        <el-table-column :label="$t('reg_codes.app')" min-width="160">
          <template #default="{ row }">{{ row.app_name || row.app_id }}</template>
        </el-table-column>
        <el-table-column :label="$t('products.price')" width="120">
          <template #default="{ row }">¥{{ formatPrice(row.price_cents) }}</template>
        </el-table-column>
        <el-table-column :label="$t('reg_codes.type')" width="100">
          <template #default="{ row }">{{ row.code_type === RegCodeType.Time ? $t('reg_codes.type_time') : $t('reg_codes.type_count') }}</template>
        </el-table-column>
        <el-table-column :label="$t('reg_codes.valid_days')" width="120">
          <template #default="{ row }">{{ row.code_type === RegCodeType.Time ? row.valid_days : '-' }}</template>
        </el-table-column>
        <el-table-column :label="$t('reg_codes.total_count')" width="120">
          <template #default="{ row }">{{ row.code_type === RegCodeType.Count ? row.total_count : '-' }}</template>
        </el-table-column>
        <el-table-column prop="status" :label="$t('products.status')" width="100">
          <template #default="{ row }">
            <el-tag :type="row.status === PlanStatus.Enabled ? 'success' : 'info'">
              {{ row.status === PlanStatus.Enabled ? $t('products.status_1') : $t('products.status_0') }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="$t('common.actions')" width="150" fixed="right" align="right">
          <template #default="{ row }">
            <el-button size="small" type="primary" plain @click="openEdit(row)">{{ $t('common.edit') }}</el-button>
            <el-button size="small" type="danger" plain @click="remove(row)">{{ $t('common.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="mt-4 flex justify-end">
        <el-pagination background layout="total, sizes, prev, pager, next, jumper" :page-sizes="[10, 20, 50, 100]"
          :page-size="pageSize" :current-page="page" :total="total" @current-change="handlePageChange"
          @size-change="handleSizeChange" />
      </div>
    </el-card>

    <el-dialog v-model="dialog.visible" :title="dialog.form.id ? $t('common.edit') : $t('common.create')" width="560px">
      <el-form label-width="120px">
        <el-form-item :label="$t('products.name')">
          <el-input v-model="dialog.form.name" />
        </el-form-item>
        <el-form-item :label="$t('reg_codes.app')">
          <el-select v-model.number="dialog.form.app_id" class="w-full" @change="onAppChange">
            <el-option v-for="app in appOptions" :key="app.id" :label="app.name" :value="app.id" />
          </el-select>
        </el-form-item>
        <el-form-item :label="$t('products.price')">
          <el-input-number v-model.number="dialog.priceYuan" :min="0.01" :precision="2" :step="1" />
        </el-form-item>
        <el-form-item :label="$t('reg_codes.type')">
          <el-radio-group v-model="dialog.form.code_type" disabled>
            <el-radio :label="RegCodeType.Time">{{ $t('reg_codes.type_time') }}</el-radio>
            <el-radio :label="RegCodeType.Count">{{ $t('reg_codes.type_count') }}</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item v-if="dialog.form.code_type === RegCodeType.Time" :label="$t('reg_codes.valid_days')">
          <el-input-number v-model.number="dialog.form.valid_days" :min="1" />
        </el-form-item>
        <el-form-item v-else :label="$t('reg_codes.total_count')">
          <el-input-number v-model.number="dialog.form.total_count" :min="1" />
        </el-form-item>
        <el-form-item :label="$t('products.status')">
          <el-switch v-model="dialog.enabled" />
        </el-form-item>
        <el-form-item :label="$t('common.remark')">
          <el-input v-model="dialog.form.description" type="textarea" :rows="3" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialog.visible = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submit">{{ $t('common.save') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { createPlan, deletePlan, fetchPlans, updatePlan } from '@/apis/payments'
import { fetchApps } from '@/apis/apps'
import type { LicensePlan, ListPlansParams, SavePlanReq } from '@/types/payments'
import { PlanStatus } from '@/types/payments'
import { RegCodeType } from '@/types/reg_codes'

const rows = ref<LicensePlan[]>([])
const appOptions = ref<{ id: number, name: string, code_type: RegCodeType }[]>([])
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const query = reactive<ListPlansParams>({ name: '', app_id: undefined, status: undefined })

const dialog = reactive({
  visible: false,
  priceYuan: 1,
  enabled: true,
  form: {
    id: undefined as number | undefined,
    app_id: 0,
    name: '',
    description: '',
    price_cents: 100,
    code_type: RegCodeType.Time,
    valid_days: 30,
    total_count: 1,
    status: PlanStatus.Enabled,
    sort_order: 0,
  },
})

type PlanForm = typeof dialog.form

function formatPrice(cents: number) { return (cents / 100).toFixed(2) }

function syncPlanTypeWithApp(appId: number) {
  const selectedApp = appOptions.value.find(app => app.id === appId)
  if (!selectedApp) return

  dialog.form.code_type = selectedApp.code_type
  if (selectedApp.code_type === RegCodeType.Time) {
    dialog.form.valid_days = dialog.form.valid_days > 0 ? dialog.form.valid_days : 30
    return
  }
  dialog.form.valid_days = 0
  dialog.form.total_count = dialog.form.total_count && dialog.form.total_count > 0 ? dialog.form.total_count : 1
}

function onAppChange(appId: number) {
  syncPlanTypeWithApp(appId)
}

async function reload() {
  const data = await fetchPlans({ ...query, page: page.value, page_size: pageSize.value })
  rows.value = data.list
  total.value = data.total
}

function reset() {
  query.name = ''
  query.app_id = undefined
  query.status = undefined
  page.value = 1
  reload()
}

function handlePageChange(p: number) { page.value = p; reload() }
function handleSizeChange(s: number) { pageSize.value = s; page.value = 1; reload() }

function openCreate() {
  const firstApp = appOptions.value[0]
  const codeType = firstApp?.code_type ?? RegCodeType.Time
  dialog.form = {
    id: undefined,
    app_id: firstApp?.id || 0,
    name: '',
    description: '',
    price_cents: 100,
    code_type: codeType,
    valid_days: codeType === RegCodeType.Time ? 30 : 0,
    total_count: 1,
    status: PlanStatus.Enabled,
    sort_order: 0,
  }
  dialog.priceYuan = 1
  dialog.enabled = true
  dialog.visible = true
}

function openEdit(row: LicensePlan) {
  dialog.form = {
    id: row.id,
    app_id: row.app_id,
    name: row.name,
    description: row.description || '',
    price_cents: row.price_cents,
    code_type: row.code_type,
    valid_days: row.valid_days,
    total_count: row.total_count ?? 1,
    status: row.status,
    sort_order: row.sort_order,
  } satisfies PlanForm
  syncPlanTypeWithApp(row.app_id)
  dialog.priceYuan = row.price_cents / 100
  dialog.enabled = row.status === PlanStatus.Enabled
  dialog.visible = true
}

async function submit() {
  const payload: SavePlanReq = {
    app_id: dialog.form.app_id,
    name: dialog.form.name,
    description: dialog.form.description,
    price_cents: Math.round(dialog.priceYuan * 100),
    code_type: dialog.form.code_type,
    valid_days: dialog.form.code_type === RegCodeType.Time ? dialog.form.valid_days : 0,
    total_count: dialog.form.code_type === RegCodeType.Count ? dialog.form.total_count : null,
    status: dialog.enabled ? PlanStatus.Enabled : PlanStatus.Disabled,
    sort_order: dialog.form.sort_order,
  }
  if (dialog.form.id) await updatePlan(dialog.form.id, payload)
  else await createPlan(payload)
  dialog.visible = false
  ElMessage.success(dialog.form.id ? 'Saved' : 'Created')
  reload()
}

async function remove(row: LicensePlan) {
  await ElMessageBox.confirm(`Delete ${row.name}?`, 'Confirm', { type: 'warning' })
  await deletePlan(row.id)
  ElMessage.success('Deleted')
  reload()
}

async function loadApps() {
  const data = await fetchApps({ page: 1, page_size: 1000 })
  appOptions.value = data.list.map(a => ({ id: a.id, name: a.name, code_type: a.code_type as RegCodeType }))
}

onMounted(async () => {
  await loadApps()
  await reload()
})
</script>
