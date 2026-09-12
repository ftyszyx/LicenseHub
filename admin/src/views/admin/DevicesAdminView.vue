<template>
  <div class="admin-list-page">
    <el-card class="admin-list-fixed" shadow="hover">
      <div class="flex items-center gap-2">
        <el-input v-model="query.device_id" :placeholder="$t('devices.device_id')" clearable class="w-56" />
        <el-input v-model.number="query.app_id" :placeholder="$t('apps.app_id')" clearable class="w-40" />
        <el-button type="primary" @click="reload">{{ $t('common.search') }}</el-button>
        <el-button @click="resetFilters">{{ $t('common.reset') }}</el-button>
      </div>
    </el-card>

    <el-card class="admin-list-panel" shadow="never">
      <el-table ref="tableRef" class="admin-list-table" :data="rows" stripe size="large" height="100%"
        @sort-change="handleSortChange">
        <el-table-column prop="id" :label="$t('common.id')" width="80" />
        <el-table-column :label="$t('devices.apptitle')" min-width="160">
          <template #default="{ row }">{{ row.app_name }} ({{ row.app_id }})</template>
        </el-table-column>
        <el-table-column prop="device_id" :label="$t('devices.device_id')" min-width="200" />
        <el-table-column prop="bound_reg_code_count" :label="$t('devices.bound_reg_code_count')" width="150"
          align="center" sortable="custom">
          <template #default="{ row }">
            <el-button v-if="row.bound_reg_code_count > 0" type="primary" link @click="openRegCodes(row)">
              {{ row.bound_reg_code_count }}
            </el-button>
            <span v-else>0</span>
          </template>
        </el-table-column>
        <el-table-column :label="$t('devices.binding_time')" min-width="180">
          <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
        </el-table-column>
        <el-table-column :label="$t('devices.expire_time')" min-width="180">
          <template #default="{ row }">{{ formatTime(row.expire_time) }}</template>
        </el-table-column>
        <el-table-column prop="remaining" :label="$t('devices.remaining')" width="120" />
      </el-table>
      <div class="admin-list-footer mt-4 flex justify-end">
        <el-pagination background layout="total, sizes, prev, pager, next, jumper" :page-sizes="[10, 20, 50, 100]" :page-size="pageSize" :current-page="page" :total="total" @current-change="handlePageChange" @size-change="handleSizeChange" />
      </div>
    </el-card>

    <el-dialog v-model="regCodeDialogVisible" :title="$t('devices.reg_codes_dialog_title', { deviceId: selectedDevice?.device_id || '' })"
      width="min(920px, calc(100vw - 32px))">
      <el-table :data="selectedDevice?.reg_codes || []" stripe max-height="520">
        <el-table-column prop="code" :label="$t('reg_codes.code')" min-width="220" />
        <el-table-column prop="status" :label="$t('reg_codes.status')" width="110">
          <template #default="{ row }">
            <el-tag :type="statusTagType(row.status)" effect="light">
              {{ $t(`reg_codes.status_${RegCodeStatus[row.status].toLowerCase()}`) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="binding_time" :label="$t('devices.binding_time')" min-width="180">
          <template #default="{ row }">{{ formatTime(row.binding_time) }}</template>
        </el-table-column>
        <el-table-column prop="expire_time" :label="$t('devices.expire_time')" min-width="180">
          <template #default="{ row }">{{ formatTime(row.expire_time) }}</template>
        </el-table-column>
      </el-table>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { fetchDevices } from '@/apis/devices'
import type { DeviceInfo, ListDevicesParams } from '@/types/app_devices'
import { RegCodeStatus } from '@/types/reg_codes'
import { formatTime } from '@/utils'

const rows = ref<DeviceInfo[]>([])
const tableRef = ref<{ clearSort: () => void } | null>(null)
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const query = reactive<ListDevicesParams>({})
const regCodeDialogVisible = ref(false)
const selectedDevice = ref<DeviceInfo | null>(null)

async function reload(){ const data = await fetchDevices({ ...query, page: page.value, page_size: pageSize.value }); rows.value = data.list; total.value = data.total }
function resetFilters(){ query.app_id=undefined; query.device_id=undefined; query.sort_by=undefined; query.sort_order=undefined; tableRef.value?.clearSort(); page.value=1; reload() }
function handlePageChange(p:number){ page.value=p; reload() }
function handleSizeChange(s:number){ pageSize.value=s; page.value=1; reload() }
function handleSortChange({ prop, order }: { prop: string, order: 'ascending' | 'descending' | null }) {
  if (prop !== 'bound_reg_code_count') return
  query.sort_by = order ? 'bound_reg_code_count' : undefined
  query.sort_order = order === 'ascending' ? 'asc' : order === 'descending' ? 'desc' : undefined
  page.value = 1
  reload()
}
function openRegCodes(row: DeviceInfo) {
  selectedDevice.value = row
  regCodeDialogVisible.value = true
}
function statusTagType(status: RegCodeStatus) {
  if (status === RegCodeStatus.Unused) return 'info'
  if (status === RegCodeStatus.Issued) return 'success'
  if (status === RegCodeStatus.binded) return 'warning'
  if (status === RegCodeStatus.Refunded || status === RegCodeStatus.Revoked) return 'danger'
  return 'info'
}

onMounted(reload)
</script>

<style scoped></style>
