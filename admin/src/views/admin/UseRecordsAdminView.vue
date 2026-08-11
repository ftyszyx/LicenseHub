<template>
  <div class="space-y-4">
    <el-card shadow="hover">
      <div class="flex items-center justify-between">
        <h2 class="text-xl font-semibold">{{ $t('use_records.title') }}</h2>
        <div class="flex items-center gap-2">
          <el-select v-model.number="query.app_id" clearable filterable class="w-48" :placeholder="$t('use_records.app')">
            <el-option v-for="opt in appOptions" :key="opt.id" :label="opt.name" :value="opt.id" />
          </el-select>
          <el-input v-model="query.device_id" clearable class="w-56" :placeholder="$t('use_records.device_id')" />
          <el-button type="primary" @click="reload">{{ $t('common.search') }}</el-button>
          <el-button @click="resetFilters">{{ $t('common.reset') }}</el-button>
        </div>
      </div>
    </el-card>

    <el-card shadow="never">
      <el-table :data="rows" stripe size="large" style="width: 100%">
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="app_name" :label="$t('use_records.app')" min-width="160">
          <template #default="{ row }">{{ row.app_name || row.app_id }}</template>
        </el-table-column>
        <el-table-column prop="device_id" :label="$t('use_records.device_id')" min-width="180" />
        <el-table-column prop="use_count" :label="$t('use_records.use_count')" width="120" />
        <el-table-column :label="$t('use_records.use_info')" min-width="260">
          <template #default="{ row }">
            <span class="break-all whitespace-pre-wrap">{{ formatUseInfo(row.use_info) }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="$t('use_records.time')" min-width="180">
          <template #default="{ row }">{{ formatTime(row.time) }}</template>
        </el-table-column>
      </el-table>

      <div class="flex justify-end mt-4">
        <el-pagination
          background
          layout="total, sizes, prev, pager, next, jumper"
          :page-sizes="[10, 20, 50, 100]"
          :page-size="pageSize"
          :current-page="page"
          :total="total"
          @current-change="handlePageChange"
          @size-change="handleSizeChange"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'

import { fetchApps } from '@/apis/apps'
import { fetchUseRecords } from '@/apis/use_records'
import type { ListUseRecordsParams, UseRecordModel } from '@/types/use_records'
import { formatTime } from '@/utils'
import { fetchAllPages } from '@/utils/pagination'

const rows = ref<UseRecordModel[]>([])
const appOptions = ref<{ id: number; name: string }[]>([])
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)

const query = reactive<ListUseRecordsParams>({
  app_id: undefined,
  device_id: '',
})

function formatUseInfo(value: any) {
  if (value == null) return ''
  if (typeof value === 'string') return value
  try {
    return JSON.stringify(value)
  } catch {
    return String(value)
  }
}

async function reload() {
  const data = await fetchUseRecords({ ...query, page: page.value, page_size: pageSize.value })
  rows.value = data.list
  total.value = data.total
}

function resetFilters() {
  query.app_id = undefined
  query.device_id = ''
  page.value = 1
  reload()
}

function handlePageChange(value: number) {
  page.value = value
  reload()
}

function handleSizeChange(value: number) {
  pageSize.value = value
  page.value = 1
  reload()
}

async function loadApps() {
  const apps = await fetchAllPages(fetchApps)
  appOptions.value = apps.map(item => ({ id: item.id, name: item.name }))
}

onMounted(async () => {
  await loadApps()
  await reload()
})
</script>

<style scoped></style>
