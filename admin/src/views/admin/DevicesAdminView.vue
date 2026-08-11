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
      <el-table class="admin-list-table" :data="rows" stripe size="large" height="100%">
        <el-table-column prop="id" :label="$t('common.id')" width="80" />
        <el-table-column :label="$t('devices.apptitle')" min-width="160">
          <template #default="{ row }">{{ row.app_name }} ({{ row.app_id }})</template>
        </el-table-column>
        <el-table-column prop="device_id" :label="$t('devices.device_id')" min-width="200" />
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
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { fetchDevices } from '@/apis/devices'
import type { DeviceInfo } from '@/types/app_devices'
import { formatTime } from '@/utils'

const rows = ref<DeviceInfo[]>([])
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const query = reactive<{ app_id?: number; device_id?: string }>({})

async function reload(){ const data = await fetchDevices({ ...query, page: page.value, page_size: pageSize.value }); rows.value = data.list; total.value = data.total }
function resetFilters(){ query.app_id=undefined; query.device_id=undefined; page.value=1; reload() }
function handlePageChange(p:number){ page.value=p; reload() }
function handleSizeChange(s:number){ pageSize.value=s; page.value=1; reload() }

onMounted(reload)
</script>

<style scoped></style>
