<template>
  <div class="admin-list-page">
    <el-card class="admin-list-fixed" shadow="hover">
      <div class="log-toolbar">
        <h2 class="text-xl font-semibold">{{ $t('system_logs.title') }}</h2>
        <div class="log-filters">
          <el-select
            v-model="selectedDate"
            class="date-select"
            :placeholder="$t('system_logs.date')"
            :empty-values="[null, undefined]"
            @change="applyFilters"
          >
            <el-option
              v-for="date in availableDates"
              :key="date"
              :label="date"
              :value="date"
            />
            <template #empty>
              <div class="py-3 text-center text-sm text-gray-500">{{ $t('system_logs.empty_dates') }}</div>
            </template>
          </el-select>

          <el-radio-group v-model="selectedLevel" @change="applyFilters">
            <el-radio-button value="">{{ $t('system_logs.all_levels') }}</el-radio-button>
            <el-radio-button value="INFO">{{ $t('system_logs.info') }}</el-radio-button>
            <el-radio-button value="WARN">{{ $t('system_logs.warning') }}</el-radio-button>
            <el-radio-button value="ERROR">{{ $t('system_logs.error') }}</el-radio-button>
          </el-radio-group>

          <el-input
            v-model="keyword"
            clearable
            class="keyword-input"
            :placeholder="$t('system_logs.keyword')"
            @keyup.enter="applyFilters"
            @clear="applyFilters"
          />
          <el-button type="primary" @click="applyFilters">{{ $t('common.search') }}</el-button>
          <el-button :icon="Refresh" :loading="loading" @click="reload">{{ $t('common.refresh') }}</el-button>
        </div>
      </div>
    </el-card>

    <el-card class="admin-list-panel" shadow="never">
      <el-table
        v-loading="loading"
        class="admin-list-table log-table"
        :data="rows"
        stripe
        size="large"
        height="100%"
        @row-click="openDetails"
      >
        <el-table-column prop="timestamp" :label="$t('system_logs.time')" width="205" />
        <el-table-column :label="$t('system_logs.level')" width="100">
          <template #default="{ row }">
            <el-tag :type="levelTagType(row.level)" effect="light">{{ row.level }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="target" :label="$t('system_logs.target')" min-width="220" show-overflow-tooltip />
        <el-table-column :label="$t('system_logs.message')" min-width="480">
          <template #default="{ row }">
            <div class="log-message">{{ row.message }}</div>
          </template>
        </el-table-column>
        <el-table-column width="56" fixed="right">
          <template #default="{ row }">
            <el-tooltip :content="$t('system_logs.details')" placement="top">
              <el-button link :icon="View" @click.stop="openDetails(row)" />
            </el-tooltip>
          </template>
        </el-table-column>
      </el-table>

      <div class="admin-list-footer mt-4 flex justify-end">
        <el-pagination
          background
          layout="total, sizes, prev, pager, next, jumper"
          :page-sizes="[20, 50, 100]"
          :page-size="pageSize"
          :current-page="page"
          :total="total"
          @current-change="handlePageChange"
          @size-change="handleSizeChange"
        />
      </div>
    </el-card>

    <el-dialog v-model="detailsVisible" :title="$t('system_logs.details')" width="min(900px, 92vw)">
      <template v-if="selectedEntry">
        <div class="detail-meta">
          <span>{{ selectedEntry.timestamp }}</span>
          <el-tag :type="levelTagType(selectedEntry.level)" effect="light">{{ selectedEntry.level }}</el-tag>
          <code>{{ selectedEntry.target }}</code>
        </div>
        <pre class="detail-message">{{ selectedEntry.message }}</pre>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { Refresh, View } from '@element-plus/icons-vue'

import { fetchSystemLogs } from '@/apis/system_logs'
import type { SystemLogEntry, SystemLogLevel } from '@/types/system_logs'

type TagType = 'primary' | 'success' | 'warning' | 'danger' | 'info'

const rows = ref<SystemLogEntry[]>([])
const availableDates = ref<string[]>([])
const selectedDate = ref('')
const selectedLevel = ref<'' | SystemLogLevel>('')
const keyword = ref('')
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const loading = ref(false)
const detailsVisible = ref(false)
const selectedEntry = ref<SystemLogEntry | null>(null)

function levelTagType(level: string): TagType {
  if (level === 'ERROR') return 'danger'
  if (level === 'WARN') return 'warning'
  if (level === 'INFO') return 'primary'
  return 'info'
}

async function reload() {
  loading.value = true
  try {
    const data = await fetchSystemLogs({
      date: selectedDate.value || undefined,
      level: selectedLevel.value || undefined,
      keyword: keyword.value.trim() || undefined,
      page: page.value,
      page_size: pageSize.value,
    })
    rows.value = data.list
    total.value = data.total
    availableDates.value = data.available_dates
    selectedDate.value = data.selected_date || ''
  } finally {
    loading.value = false
  }
}

function applyFilters() {
  page.value = 1
  void reload()
}

function handlePageChange(value: number) {
  page.value = value
  void reload()
}

function handleSizeChange(value: number) {
  pageSize.value = value
  page.value = 1
  void reload()
}

function openDetails(entry: SystemLogEntry) {
  selectedEntry.value = entry
  detailsVisible.value = true
}

onMounted(() => {
  void reload()
})
</script>

<style scoped>
.log-toolbar,
.log-filters,
.detail-meta {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.log-toolbar {
  justify-content: space-between;
  flex-wrap: wrap;
}

.log-filters {
  flex-wrap: wrap;
  justify-content: flex-end;
}

.date-select {
  width: 150px;
}

.keyword-input {
  width: min(260px, 70vw);
}

.log-table :deep(.el-table__row) {
  cursor: pointer;
}

.log-message {
  display: -webkit-box;
  overflow: hidden;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}

.detail-meta {
  margin-bottom: 0.75rem;
  flex-wrap: wrap;
  color: #475569;
}

.detail-message {
  max-height: 60vh;
  margin: 0;
  overflow: auto;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  background: #f8fafc;
  padding: 1rem;
  color: #0f172a;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 13px;
  line-height: 1.6;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

@media (max-width: 900px) {
  .log-filters {
    width: 100%;
    justify-content: flex-start;
  }
}
</style>
