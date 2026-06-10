<template>
  <div class="space-y-4">
    <el-card shadow="hover">
      <div class="flex items-center justify-between">
        <h2 class="text-xl font-semibold">{{ $t('apps.title') }}</h2>
        <div class="flex items-center gap-2">
          <el-input v-model="query.name" :placeholder="$t('common.search_by_name')" clearable class="w-64" />
          <el-button type="primary" @click="reload">
            <el-icon class="mr-1"><Search /></el-icon>
            {{ $t('common.search') }}
          </el-button>
          <el-button @click="resetFilters">
            <el-icon class="mr-1"><Refresh /></el-icon>
            {{ $t('common.reset') }}
          </el-button>
          <el-button type="success" @click="openCreate">
            <el-icon class="mr-1"><Plus /></el-icon>
            {{ $t('common.new') }}
          </el-button>
        </div>
      </div>
    </el-card>

    <el-card shadow="never">
      <el-table v-loading="loading" :data="apps" stripe size="large" style="width: 100%">
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column :label="$t('apps.name')" min-width="220">
          <template #default="{ row }">
            <div class="flex flex-col leading-tight">
              <span class="font-medium">{{ row.name }}</span>
              <span class="text-xs text-gray-500">{{ row.app_id }}</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column :label="$t('apps.version')" width="150">
          <template #default="{ row }">
            <div class="flex flex-col leading-tight">
              <span class="font-medium">{{ row.app_vername || '-' }}</span>
              <span class="text-xs text-gray-500">{{ $t('apps.version_code') }}: {{ row.app_vercode ?? '-' }}</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column :label="$t('apps.valid_key')" min-width="240">
          <template #default="{ row }">
            <div class="flex items-center gap-2">
              <span class="break-all text-gray-600">{{ row.app_valid_key }}</span>
              <el-button size="small" @click="copyKey(row.app_valid_key)">{{ $t('common.copy') }}</el-button>
            </div>
          </template>
        </el-table-column>
        <el-table-column :label="$t('apps.trial_days')" width="120">
          <template #default="{ row }">{{ row.trial_days }}</template>
        </el-table-column>
        <el-table-column :label="$t('apps.trial_num')" width="120">
          <template #default="{ row }">{{ row.trial_num }}</template>
        </el-table-column>
        <el-table-column :label="$t('apps.code_type')" width="120">
          <template #default="{ row }">
            <span v-if="row.code_type === RegCodeType.Time">{{ $t('reg_codes.type_time') }}</span>
            <span v-else-if="row.code_type === RegCodeType.Count">{{ $t('reg_codes.type_count') }}</span>
            <span v-else>{{ row.code_type }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="$t('apps.links')" min-width="160">
          <template #default="{ row }">
            <div class="flex flex-col items-start gap-1">
              <el-link v-if="row.app_download_url" type="primary" :href="row.app_download_url" target="_blank">
                {{ $t('apps.download') }}
              </el-link>
              <el-link v-if="row.app_res_url" type="primary" :href="row.app_res_url" target="_blank">
                {{ $t('apps.resource') }}
              </el-link>
              <span v-if="!row.app_download_url && !row.app_res_url" class="text-gray-400">-</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column :label="$t('apps.manifest_urls')" width="150">
          <template #default="{ row }">
            <div v-if="manifestUrlCount(row)" class="flex items-center gap-2">
              <el-button size="small" plain @click="openManifestUrls(row)">
                <el-icon class="mr-1"><View /></el-icon>
                {{ $t('apps.view_manifest_urls') }}
              </el-button>
              <el-tag size="small">{{ manifestUrlCount(row) }}</el-tag>
            </div>
            <span v-else class="text-gray-400">-</span>
          </template>
        </el-table-column>
        <el-table-column prop="status" :label="$t('apps.status')" width="100">
          <template #default="{ row }">
            <el-tag :type="row.status === 1 ? 'success' : 'info'">
              {{ row.status === 1 ? $t('apps.enabled') : $t('apps.disabled') }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="$t('apps.created')" min-width="180">
          <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
        </el-table-column>
        <el-table-column :label="$t('apps.updated')" min-width="180">
          <template #default="{ row }">{{ formatTime(row.updated_at) }}</template>
        </el-table-column>
        <el-table-column :label="$t('common.actions')" width="460" fixed="right" align="right">
          <template #default="{ row }">
            <div class="flex flex-wrap justify-end gap-1">
              <el-button size="small" type="success" plain @click="openBuyPage(row)">
                <el-icon class="mr-1"><ShoppingCart /></el-icon>
                {{ $t('apps.buy_page') }}
              </el-button>
              <el-button size="small" type="warning" plain @click="openSync(row)">
                <el-icon class="mr-1"><Upload /></el-icon>
                {{ $t('apps.sync_version') }}
              </el-button>
              <el-button size="small" type="primary" plain @click="openSyncLogs(row)">
                <el-icon class="mr-1"><Tickets /></el-icon>
                {{ $t('apps.sync_logs') }}
              </el-button>
              <el-button size="small" @click="openEdit(row)">
                <el-icon class="mr-1"><Edit /></el-icon>
                {{ $t('common.edit') }}
              </el-button>
              <el-button size="small" type="danger" @click="confirmDelete(row)">
                <el-icon class="mr-1"><Delete /></el-icon>
                {{ $t('common.delete') }}
              </el-button>
            </div>
          </template>
        </el-table-column>
      </el-table>

      <div class="mt-4 flex justify-end">
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

    <el-dialog v-model="dialog.visible" :title="dialog.mode === 'create' ? $t('apps.create_title') : $t('apps.edit_title')" width="960px">
      <el-form ref="formRef" :model="form" label-width="104px" :rules="rules">
        <el-tabs v-model="appDialogTab" type="card" stretch class="app-edit-tabs">
          <el-tab-pane :label="$t('apps.basic_info')" name="basic">
            <div class="app-edit-tab-panel">
              <div class="rounded border border-slate-100 bg-slate-50 px-4 pt-4">
                <div class="grid grid-cols-2 gap-x-5">
                  <el-form-item :label="$t('apps.name')" prop="name">
                    <el-input v-model="form.name" />
                  </el-form-item>
                  <el-form-item :label="$t('apps.app_id')" prop="app_id">
                    <el-input v-model="form.app_id">
                      <template #append>
                        <el-button @click="genAppId">{{ $t('common.generate') }}</el-button>
                      </template>
                    </el-input>
                  </el-form-item>
                  <el-form-item :label="$t('apps.valid_key')" prop="app_valid_key" class="col-span-2">
                    <el-input v-model="form.app_valid_key">
                      <template #append>
                        <el-button @click="genAppKey">{{ $t('common.generate') }}</el-button>
                      </template>
                    </el-input>
                  </el-form-item>
                  <el-form-item :label="$t('apps.code_type')" prop="code_type">
                    <el-select v-model.number="form.code_type" class="w-full" @change="onCodeTypeChange">
                      <el-option :label="$t('reg_codes.type_time')" :value="RegCodeType.Time" />
                      <el-option :label="$t('reg_codes.type_count')" :value="RegCodeType.Count" />
                    </el-select>
                  </el-form-item>
                  <el-form-item v-if="form.code_type === RegCodeType.Time" :label="$t('apps.trial_days')" prop="trial_days">
                    <el-input-number v-model.number="form.trial_days" :min="0" class="w-full" />
                  </el-form-item>
                  <el-form-item v-if="form.code_type === RegCodeType.Count" :label="$t('apps.trial_num')" prop="trial_num">
                    <el-input-number v-model.number="form.trial_num" :min="0" class="w-full" />
                  </el-form-item>
                  <el-form-item :label="$t('apps.sort_order')" prop="sort_order">
                    <el-input-number v-model="form.sort_order" :min="0" class="w-full" />
                  </el-form-item>
                  <el-form-item :label="$t('apps.status')" prop="status">
                    <el-select v-model="form.status" class="w-full">
                      <el-option :label="$t('apps.enabled')" :value="1" />
                      <el-option :label="$t('apps.disabled')" :value="0" />
                    </el-select>
                  </el-form-item>
                </div>
              </div>
            </div>
          </el-tab-pane>

          <el-tab-pane :label="$t('apps.version_sync_info')" name="version">
            <div class="app-edit-tab-panel">
              <div class="rounded border border-slate-100 bg-slate-50 px-4 pt-4">
                <div class="grid grid-cols-2 gap-x-5">
                  <el-form-item :label="$t('apps.version_name')" prop="app_vername">
                    <el-input v-model="form.app_vername" />
                  </el-form-item>
                  <el-form-item :label="$t('apps.version_code')" prop="app_vercode">
                    <el-input-number v-model.number="form.app_vercode" :min="0" class="w-full" />
                  </el-form-item>
                  <el-form-item :label="$t('apps.download_url')" prop="app_download_url">
                    <el-input v-model="form.app_download_url" />
                  </el-form-item>
                  <el-form-item :label="$t('apps.resource_url')" prop="app_res_url">
                    <el-input v-model="form.app_res_url" />
                  </el-form-item>
                  <el-form-item :label="$t('apps.update_info')" prop="app_update_info" class="col-span-2">
                    <el-input v-model="form.app_update_info" type="textarea" :rows="4" resize="vertical" />
                  </el-form-item>
                </div>
              </div>
            </div>
          </el-tab-pane>

          <el-tab-pane :label="$t('apps.manifest_extra')" name="extra">
            <div class="app-edit-tab-panel">
              <div class="rounded border border-slate-200 bg-white">
                <div class="grid grid-cols-12 gap-3 border-b border-slate-100 bg-slate-50 px-3 py-2 text-xs font-medium text-slate-500">
                  <span class="col-span-3">{{ $t('apps.manifest_extra_key') }}</span>
                  <span class="col-span-2">{{ $t('apps.manifest_extra_type') }}</span>
                  <span class="col-span-6">{{ $t('apps.manifest_extra_value') }}</span>
                  <span class="col-span-1" />
                </div>
                <div class="space-y-2 p-3">
                  <div
                    v-for="row in manifestExtraRows"
                    :key="row.id"
                    class="grid grid-cols-12 items-start gap-3"
                  >
                    <el-input v-model="row.key" class="col-span-3" :placeholder="$t('apps.manifest_extra_key_placeholder')" clearable />
                    <el-select v-model="row.type" class="col-span-2" @change="onManifestExtraTypeChange(row)">
                      <el-option :label="$t('apps.manifest_type_string')" value="string" />
                      <el-option :label="$t('apps.manifest_type_number')" value="number" />
                      <el-option :label="$t('apps.manifest_type_boolean')" value="boolean" />
                      <el-option :label="$t('apps.manifest_type_json')" value="json" />
                    </el-select>
                    <el-select v-if="row.type === 'boolean'" v-model="row.value" class="col-span-6">
                      <el-option :label="$t('apps.manifest_boolean_true')" value="true" />
                      <el-option :label="$t('apps.manifest_boolean_false')" value="false" />
                    </el-select>
                    <el-input
                      v-else-if="row.type === 'json'"
                      v-model="row.value"
                      class="col-span-6"
                      type="textarea"
                      :rows="2"
                      resize="vertical"
                      :placeholder="$t('apps.manifest_extra_json_placeholder')"
                    />
                    <el-input v-else v-model="row.value" class="col-span-6" :placeholder="$t('apps.manifest_extra_value_placeholder')" clearable />
                    <el-button class="col-span-1 w-full" :disabled="manifestExtraRows.length === 1" @click="removeManifestExtraRow(row.id)">
                      <el-icon><Delete /></el-icon>
                    </el-button>
                  </div>
                  <el-button @click="addManifestExtraRow">
                    <el-icon class="mr-1"><Plus /></el-icon>
                    {{ $t('apps.add_manifest_extra') }}
                  </el-button>
                </div>
              </div>
            </div>
          </el-tab-pane>
        </el-tabs>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="dialog.visible = false">{{ $t('common.cancel') }}</el-button>
          <el-button type="primary" @click="submitForm(formRef)">{{ $t('common.confirm') }}</el-button>
        </span>
      </template>
    </el-dialog>

    <el-dialog v-model="syncDialog.visible" :title="$t('apps.sync_version')" width="820px">
      <div class="space-y-4">
        <div v-if="syncDialog.app" class="rounded border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-600">
          {{ syncDialog.app.name }} / {{ syncDialog.app.app_vername }} ({{ syncDialog.app.app_vercode }})
        </div>

        <div>
          <div class="mb-2 flex items-center justify-between">
            <span class="text-sm font-medium text-slate-700">{{ $t('apps.enabled_channels') }}</span>
            <el-button size="small" :loading="syncDialog.loadingChannels" @click="loadEnabledChannels">{{ $t('common.refresh') }}</el-button>
          </div>
          <div v-loading="syncDialog.loadingChannels" class="min-h-16 rounded border border-slate-200 p-3">
            <el-checkbox-group v-if="enabledChannels.length" v-model="syncDialog.channelIds" class="grid grid-cols-2 gap-2">
              <el-checkbox v-for="channel in enabledChannels" :key="channel.id" :value="channel.id" class="min-w-0">
                <span class="truncate">{{ channelName(channel) }}</span>
              </el-checkbox>
            </el-checkbox-group>
            <el-empty v-else :description="$t('apps.no_enabled_channels')" :image-size="72" />
          </div>
          <div class="mt-2 text-xs text-slate-500">{{ $t('apps.sync_all_enabled_hint') }}</div>
        </div>

        <div class="flex items-center gap-2">
          <el-button :loading="syncDialog.manifestLoading" @click="previewManifest">{{ $t('apps.preview_manifest') }}</el-button>
          <el-button type="warning" :loading="syncDialog.syncing" @click="runSync">{{ $t('apps.sync_version') }}</el-button>
        </div>

        <div v-if="syncDialog.manifest" class="space-y-2">
          <div class="text-sm font-medium text-slate-700">{{ $t('apps.manifest_preview') }}</div>
          <pre class="max-h-64 overflow-auto rounded border border-slate-700 bg-slate-950 p-3 font-mono text-xs leading-5 text-white shadow-inner">{{ formatJson(syncDialog.manifest) }}</pre>
        </div>

        <div v-if="syncDialog.result" class="space-y-2">
          <div class="text-sm font-medium text-slate-700">{{ $t('apps.sync_result') }}</div>
          <pre class="max-h-64 overflow-auto rounded border border-slate-700 bg-slate-950 p-3 font-mono text-xs leading-5 text-white shadow-inner">{{ formatJson(syncDialog.result) }}</pre>
        </div>
      </div>
      <template #footer>
        <el-button @click="syncDialog.visible = false">{{ $t('common.cancel') }}</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="logsDialog.visible" :title="$t('apps.sync_logs')" width="1080px">
      <div class="space-y-4">
        <div v-if="logsDialog.app" class="rounded border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-600">
          {{ logsDialog.app.name }} / {{ logsDialog.app.app_vername }} ({{ logsDialog.app.app_vercode }})
        </div>

        <el-table v-loading="logsDialog.loading" :data="logsDialog.rows" stripe size="small" style="width: 100%">
          <el-table-column prop="id" label="ID" width="80" />
          <el-table-column :label="$t('apps.sync_log_provider')" width="130">
            <template #default="{ row }">{{ storageProviderLabel(row.provider) }}</template>
          </el-table-column>
          <el-table-column :label="$t('apps.sync_log_status')" width="110">
            <template #default="{ row }">
              <el-tag :type="syncLogStatusTag(row.status)">{{ syncLogStatusLabel(row.status) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column :label="$t('apps.sync_log_object_key')" min-width="220">
            <template #default="{ row }">
              <span class="break-all text-slate-600">{{ row.object_key }}</span>
            </template>
          </el-table-column>
          <el-table-column :label="$t('apps.sync_log_public_url')" min-width="240">
            <template #default="{ row }">
              <el-link v-if="row.public_url" type="primary" :href="row.public_url" target="_blank" class="break-all">
                {{ row.public_url }}
              </el-link>
              <span v-else>-</span>
            </template>
          </el-table-column>
          <el-table-column :label="$t('apps.sync_log_error')" min-width="240">
            <template #default="{ row }">
              <el-popover
                v-if="row.error_message"
                placement="top"
                trigger="click"
                width="520"
              >
                <template #reference>
                  <el-button link type="danger" class="max-w-full">
                    <span class="inline-block max-w-[220px] truncate align-bottom text-left text-xs">
                      {{ row.error_message }}
                    </span>
                  </el-button>
                </template>
                <pre class="max-h-80 overflow-auto whitespace-pre-wrap break-words rounded bg-slate-950 p-3 font-mono text-xs leading-5 text-white">{{ row.error_message }}</pre>
              </el-popover>
              <span v-else>-</span>
            </template>
          </el-table-column>
          <el-table-column :label="$t('apps.sync_log_created_at')" width="170">
            <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
          </el-table-column>
          <el-table-column :label="$t('common.actions')" width="110" fixed="right" align="right">
            <template #default="{ row }">
              <el-button size="small" plain @click="showLogManifest(row)">{{ $t('apps.preview_manifest') }}</el-button>
            </template>
          </el-table-column>
        </el-table>

        <div class="flex justify-end">
          <el-pagination
            background
            layout="total, prev, pager, next"
            :page-size="logsDialog.pageSize"
            :current-page="logsDialog.page"
            :total="logsDialog.total"
            @current-change="handleLogsPageChange"
          />
        </div>
      </div>
      <template #footer>
        <el-button @click="logsDialog.visible = false">{{ $t('common.cancel') }}</el-button>
        <el-button :loading="logsDialog.loading" @click="loadSyncLogs">{{ $t('common.refresh') }}</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="manifestUrlsDialog.visible" :title="$t('apps.manifest_urls')" width="880px">
      <div class="space-y-4">
        <div v-if="manifestUrlsDialog.app" class="rounded border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-600">
          {{ manifestUrlsDialog.app.name }} / {{ manifestUrlsDialog.app.app_vername }} ({{ manifestUrlsDialog.app.app_vercode }})
        </div>

        <el-table :data="manifestUrlsDialog.urls" stripe size="small" style="width: 100%">
          <el-table-column :label="$t('apps.sync_log_provider')" width="130">
            <template #default="{ row }">{{ storageProviderLabel(row.provider) }}</template>
          </el-table-column>
          <el-table-column :label="$t('storage_channels.channel')" min-width="150">
            <template #default="{ row }">{{ row.channel_name }}</template>
          </el-table-column>
          <el-table-column :label="$t('apps.sync_log_public_url')" min-width="300">
            <template #default="{ row }">
              <el-link type="primary" :href="row.public_url" target="_blank" class="break-all">
                {{ row.public_url }}
              </el-link>
            </template>
          </el-table-column>
          <el-table-column :label="$t('apps.sync_log_created_at')" width="170">
            <template #default="{ row }">{{ formatTime(row.synced_at) }}</template>
          </el-table-column>
          <el-table-column :label="$t('common.actions')" width="110" fixed="right" align="right">
            <template #default="{ row }">
              <el-button size="small" plain @click="copyManifestUrl(row.public_url)">
                <el-icon class="mr-1"><DocumentCopy /></el-icon>
                {{ $t('common.copy') }}
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
      <template #footer>
        <el-button @click="manifestUrlsDialog.visible = false">{{ $t('common.cancel') }}</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="manifestDialog.visible" :title="$t('apps.manifest_preview')" width="760px">
      <pre class="max-h-[520px] overflow-auto rounded border border-slate-700 bg-slate-950 p-3 font-mono text-xs leading-5 text-white shadow-inner">{{ formatJson(manifestDialog.manifest) }}</pre>
      <template #footer>
        <el-button @click="manifestDialog.visible = false">{{ $t('common.cancel') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox, type FormInstance, type FormRules } from 'element-plus'
import { createApp, deleteApp, fetchApps, updateApp } from '@/apis/apps'
import { fetchStorageChannels, fetchVersionManifest, fetchVersionSyncLogs, syncAppVersion } from '@/apis/storage'
import type { AddAppReq, AppManifestUrl, AppModel, UpdateAppReq } from '@/types/apps'
import { RegCodeType } from '@/types/reg_codes'
import { RoutePath } from '@/types/route'
import {
  StorageChannelStatus,
  type StorageChannel,
  type StorageProvider,
  type SyncVersionResp,
  type VersionManifest,
  type VersionSyncLog,
} from '@/types/storage'

type ManifestExtraType = 'string' | 'number' | 'boolean' | 'json'
type ManifestExtraRow = {
  id: number
  key: string
  value: string
  type: ManifestExtraType
}
type AppDialogTab = 'basic' | 'version' | 'extra'

const apps = ref<AppModel[]>([])
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const loading = ref(false)
const router = useRouter()
const { t } = useI18n()

const query = reactive({
  name: '' as string | undefined,
})

const dialog = reactive({ visible: false, mode: 'create' as 'create' | 'edit', editingId: undefined as number | undefined })
const appDialogTab = ref<AppDialogTab>('basic')
const emptyForm: AddAppReq = {
  name: '',
  app_id: '',
  app_vername: '1.0.0',
  app_vercode: 1,
  app_download_url: '',
  app_res_url: '',
  app_update_info: '',
  code_type: RegCodeType.Time,
  app_valid_key: '',
  trial_days: 0,
  trial_num: 0,
  sort_order: 0,
  status: 1,
}
const form = reactive<AddAppReq>({ ...emptyForm })
const formRef = ref<FormInstance>()
const manifestExtraRows = ref<ManifestExtraRow[]>([])
let manifestExtraRowId = 0

const rules = reactive<FormRules<AddAppReq>>({
  name: [{ required: true, message: t('apps.input_name'), trigger: 'blur' }],
  app_id: [{ required: true, message: t('apps.input_app_id'), trigger: 'blur' }],
  app_valid_key: [{ required: true, message: t('apps.input_valid_key'), trigger: 'blur' }],
  app_vername: [{ required: true, message: t('apps.input_version_name'), trigger: 'blur' }],
  app_vercode: [{ required: true, message: t('apps.input_version_code'), trigger: 'blur' }],
})

const appDialogFieldTabs: Record<string, AppDialogTab> = {
  name: 'basic',
  app_id: 'basic',
  app_valid_key: 'basic',
  app_vername: 'version',
  app_vercode: 'version',
}

const enabledChannels = ref<StorageChannel[]>([])
const syncDialog = reactive({
  visible: false,
  app: undefined as AppModel | undefined,
  channelIds: [] as number[],
  loadingChannels: false,
  syncing: false,
  manifestLoading: false,
  manifest: undefined as VersionManifest | undefined,
  result: undefined as SyncVersionResp | undefined,
})

const logsDialog = reactive({
  visible: false,
  app: undefined as AppModel | undefined,
  rows: [] as VersionSyncLog[],
  loading: false,
  page: 1,
  pageSize: 10,
  total: 0,
})

const manifestDialog = reactive({
  visible: false,
  manifest: undefined as VersionManifest | undefined,
})

const manifestUrlsDialog = reactive({
  visible: false,
  app: undefined as AppModel | undefined,
  urls: [] as AppManifestUrl[],
})

const reload = async () => {
  loading.value = true
  try {
    const data = await fetchApps({ page: page.value, page_size: pageSize.value, name: query.name || undefined })
    apps.value = data.list
    total.value = data.total
  } finally {
    loading.value = false
  }
}

const handlePageChange = async (p: number) => {
  page.value = p
  await reload()
}

const handleSizeChange = async (s: number) => {
  pageSize.value = s
  page.value = 1
  await reload()
}

const resetFilters = async () => {
  query.name = ''
  page.value = 1
  await reload()
}

const onCodeTypeChange = () => {
  if (form.code_type === RegCodeType.Time) {
    form.trial_num = 0
  } else if (form.code_type === RegCodeType.Count) {
    form.trial_days = 0
  }
}

function createManifestExtraRow(value?: Partial<ManifestExtraRow>): ManifestExtraRow {
  manifestExtraRowId += 1
  return {
    id: manifestExtraRowId,
    key: value?.key ?? '',
    value: value?.value ?? '',
    type: value?.type ?? 'string',
  }
}

function resetManifestExtraRows(extra?: Record<string, unknown> | null) {
  const rows = Object.entries(extra ?? {}).map(([key, value]) => {
    const valueType = typeof value
    if (valueType === 'number') {
      return createManifestExtraRow({ key, type: 'number', value: String(value) })
    }
    if (valueType === 'boolean') {
      return createManifestExtraRow({ key, type: 'boolean', value: value ? 'true' : 'false' })
    }
    if (valueType === 'string') {
      return createManifestExtraRow({ key, type: 'string', value: String(value) })
    }
    return createManifestExtraRow({ key, type: 'json', value: JSON.stringify(value, null, 2) })
  })

  manifestExtraRows.value = rows.length ? rows : [createManifestExtraRow()]
}

function addManifestExtraRow() {
  manifestExtraRows.value.push(createManifestExtraRow())
}

function removeManifestExtraRow(id: number) {
  manifestExtraRows.value = manifestExtraRows.value.filter(row => row.id !== id)
  if (!manifestExtraRows.value.length) {
    addManifestExtraRow()
  }
}

function onManifestExtraTypeChange(row: ManifestExtraRow) {
  if (row.type === 'boolean' && row.value !== 'true' && row.value !== 'false') {
    row.value = 'false'
  }
}

function buildManifestExtra() {
  const extra: Record<string, unknown> = {}
  for (const row of manifestExtraRows.value) {
    const key = row.key.trim()
    if (!key) continue

    if (row.type === 'number') {
      if (!row.value.trim()) {
        ElMessage.error(t('apps.invalid_manifest_extra_number', { key }) as string)
        return undefined
      }
      const numberValue = Number(row.value)
      if (!Number.isFinite(numberValue)) {
        ElMessage.error(t('apps.invalid_manifest_extra_number', { key }) as string)
        return undefined
      }
      extra[key] = numberValue
      continue
    }

    if (row.type === 'boolean') {
      extra[key] = row.value === 'true'
      continue
    }

    if (row.type === 'json') {
      try {
        extra[key] = JSON.parse(row.value)
      } catch (_error) {
        ElMessage.error(t('apps.invalid_manifest_extra_json', { key }) as string)
        return undefined
      }
      continue
    }

    extra[key] = row.value
  }

  return extra
}

const openCreate = () => {
  Object.assign(form, emptyForm)
  resetManifestExtraRows()
  onCodeTypeChange()
  dialog.mode = 'create'
  dialog.editingId = undefined
  appDialogTab.value = 'basic'
  dialog.visible = true
  formRef.value?.clearValidate()
}

const openEdit = (row: AppModel) => {
  dialog.mode = 'edit'
  dialog.editingId = row.id
  Object.assign(form, {
    name: row.name,
    app_id: row.app_id,
    app_vername: row.app_vername || '1.0.0',
    app_vercode: row.app_vercode || 1,
    app_download_url: row.app_download_url || '',
    app_res_url: row.app_res_url || '',
    app_update_info: row.app_update_info || '',
    code_type: row.code_type ?? RegCodeType.Time,
    app_valid_key: row.app_valid_key || '',
    trial_days: row.trial_days ?? 0,
    trial_num: row.trial_num ?? 0,
    sort_order: row.sort_order,
    status: row.status,
  })
  resetManifestExtraRows(row.manifest_extra)
  onCodeTypeChange()
  appDialogTab.value = 'basic'
  dialog.visible = true
  formRef.value?.clearValidate()
}

const openBuyPage = (row: AppModel) => {
  router.push(`${RoutePath.Products}/${row.id}`)
}

const submitForm = async (currentFormRef: FormInstance | undefined) => {
  if (!currentFormRef) return
  let valid = false
  try {
    valid = await currentFormRef.validate()
  } catch (_error) {
    valid = false
  }
  if (!valid) {
    const invalidField = currentFormRef.fields.find(field => field.validateState === 'error')
    const invalidTab = invalidField?.propString ? appDialogFieldTabs[invalidField.propString] : undefined
    if (invalidTab) {
      appDialogTab.value = invalidTab
    }
    ElMessage.error(t('common.please_check_form') as string)
    return
  }

  const manifestExtra = buildManifestExtra()
  if (!manifestExtra) {
    appDialogTab.value = 'extra'
    return
  }

  const payload: AddAppReq = { ...form, manifest_extra: manifestExtra }
  if (dialog.mode === 'create') {
    await createApp(payload)
  } else if (dialog.editingId != null) {
    await updateApp(dialog.editingId, payload as UpdateAppReq)
  }
  dialog.visible = false
  await reload()
  ElMessage.success(t('common.saved') as string)
}

function genAppKey() {
  const ts = Date.now().toString(36)
  const rand = Math.random().toString(36).slice(2)
  form.app_valid_key = `${ts}-${rand}`
}

function genAppId() {
  const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'
  const len = 12
  if (typeof crypto !== 'undefined' && 'getRandomValues' in crypto) {
    const array = new Uint8Array(len)
    crypto.getRandomValues(array)
    form.app_id = Array.from(array, n => alphabet[n % alphabet.length]).join('')
  } else {
    let s = ''
    for (let i = 0; i < len; i++) {
      s += alphabet[Math.floor(Math.random() * alphabet.length)]
    }
    form.app_id = s
  }
}

const confirmDelete = async (row: AppModel) => {
  try {
    await ElMessageBox.confirm(t('apps.delete_confirm', { name: row.name }) as string, t('common.confirm') as string, {
      type: 'warning',
      confirmButtonText: t('common.delete') as string,
      cancelButtonText: t('common.cancel') as string,
    })
    await deleteApp(row.id)
    ElMessage.success(t('common.deleted') as string)
    await reload()
  } catch (_error) {
    // cancel
  }
}

function storageProviderLabel(provider: StorageProvider) {
  return provider === 'aliyun_oss' ? t('storage_channels.provider_aliyun_oss') : t('storage_channels.provider_cloudflare_r2')
}

function channelName(channel: StorageChannel) {
  if (channel.name) return channel.name
  const bucket = channel.config?.bucket || '-'
  const prefix = channel.config?.prefix ? `/${channel.config.prefix}` : ''
  return `${storageProviderLabel(channel.provider)} - ${bucket}${prefix}`
}

async function loadEnabledChannels() {
  syncDialog.loadingChannels = true
  try {
    const all: StorageChannel[] = []
    let currentPage = 1
    let channelTotal = 0
    do {
      const data = await fetchStorageChannels({
        page: currentPage,
        page_size: 100,
        status: StorageChannelStatus.Enabled,
      })
      all.push(...data.list)
      channelTotal = data.total
      currentPage += 1
    } while (all.length < channelTotal)

    enabledChannels.value = all
    syncDialog.channelIds = syncDialog.channelIds.filter(id => all.some(channel => channel.id === id))
  } finally {
    syncDialog.loadingChannels = false
  }
}

async function openSync(row: AppModel) {
  syncDialog.app = row
  syncDialog.channelIds = []
  syncDialog.manifest = undefined
  syncDialog.result = undefined
  syncDialog.visible = true
  await loadEnabledChannels()
}

async function previewManifest() {
  if (!syncDialog.app) return
  syncDialog.manifestLoading = true
  try {
    syncDialog.manifest = await fetchVersionManifest(syncDialog.app.id)
  } finally {
    syncDialog.manifestLoading = false
  }
}

async function runSync() {
  if (!syncDialog.app) return
  syncDialog.syncing = true
  try {
    const payload = syncDialog.channelIds.length ? { channel_ids: [...syncDialog.channelIds] } : {}
    syncDialog.result = await syncAppVersion(syncDialog.app.id, payload)
    syncDialog.app.manifest_urls = mergeManifestUrls(syncDialog.app.manifest_urls, syncDialog.result)
    showSyncResultMessage(syncDialog.result)
  } finally {
    syncDialog.syncing = false
  }
}

function showSyncResultMessage(result: SyncVersionResp) {
  const total = result.results.length
  const success = result.results.filter(item => item.success).length
  const params = { success, total }

  if (!total) {
    ElMessage.warning(t('apps.sync_no_result') as string)
  } else if (success === total) {
    ElMessage.success(t('apps.sync_all_success', params) as string)
  } else if (success > 0) {
    ElMessage.warning(t('apps.sync_partial_failed', params) as string)
  } else {
    ElMessage.error(t('apps.sync_all_failed', params) as string)
  }
}

function mergeManifestUrls(existing: AppManifestUrl[] | undefined, result: SyncVersionResp) {
  const urls = [...(existing ?? [])]
  for (const item of result.results) {
    if (!item.success || !item.public_url) continue
    const next: AppManifestUrl = {
      channel_id: item.channel_id,
      channel_name: item.channel_name,
      provider: item.provider,
      public_url: item.public_url,
      object_key: item.object_key,
      synced_at: new Date().toISOString(),
    }
    const index = urls.findIndex(url => url.channel_id === item.channel_id)
    if (index >= 0) urls[index] = next
    else urls.push(next)
  }
  return urls.sort((a, b) => a.channel_id - b.channel_id)
}

function manifestUrlCount(row: AppModel) {
  return row.manifest_urls?.length ?? 0
}

function openManifestUrls(row: AppModel) {
  manifestUrlsDialog.app = row
  manifestUrlsDialog.urls = [...(row.manifest_urls ?? [])]
  manifestUrlsDialog.visible = true
}

async function openSyncLogs(row: AppModel) {
  logsDialog.app = row
  logsDialog.page = 1
  logsDialog.rows = []
  logsDialog.total = 0
  logsDialog.visible = true
  await loadSyncLogs()
}

async function loadSyncLogs() {
  if (!logsDialog.app) return
  logsDialog.loading = true
  try {
    const data = await fetchVersionSyncLogs({
      app_id: logsDialog.app.id,
      page: logsDialog.page,
      page_size: logsDialog.pageSize,
    })
    logsDialog.rows = data.list
    logsDialog.total = data.total
  } finally {
    logsDialog.loading = false
  }
}

async function handleLogsPageChange(pageValue: number) {
  logsDialog.page = pageValue
  await loadSyncLogs()
}

function syncLogStatusLabel(status: number) {
  if (status === 1) return t('apps.sync_log_success')
  if (status === 2) return t('apps.sync_log_failed')
  return t('apps.sync_log_pending')
}

function syncLogStatusTag(status: number) {
  if (status === 1) return 'success'
  if (status === 2) return 'danger'
  return 'info'
}

function showLogManifest(row: VersionSyncLog) {
  manifestDialog.manifest = row.manifest
  manifestDialog.visible = true
}

const formatTime = (iso: string) => {
  if (!iso) return '-'
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return iso
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

function formatJson(value: unknown) {
  return JSON.stringify(value, null, 2)
}

async function copyKey(key: string) {
  try {
    await navigator.clipboard.writeText(key || '')
    ElMessage.success(t('common.copied') as string)
  } catch (_error) {
    ElMessage.error(t('common.copy_failed') as string)
  }
}

async function copyManifestUrl(url?: string | null) {
  if (!url) return
  try {
    await navigator.clipboard.writeText(url)
    ElMessage.success(t('common.copied') as string)
  } catch (_error) {
    ElMessage.error(t('common.copy_failed') as string)
  }
}

onMounted(reload)
</script>

<style scoped>
.app-edit-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
}

.app-edit-tabs :deep(.el-tabs__item) {
  height: 38px;
  font-weight: 500;
}

.app-edit-tab-panel {
  max-height: 62vh;
  overflow-y: auto;
  padding-right: 8px;
}
</style>
