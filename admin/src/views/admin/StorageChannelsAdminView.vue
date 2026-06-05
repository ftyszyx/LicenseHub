<template>
  <div class="space-y-4">
    <el-card shadow="hover">
      <div class="flex items-center justify-between gap-3">
        <h2 class="text-xl font-semibold">{{ $t('storage_channels.title') }}</h2>
        <div class="flex items-center gap-2">
          <el-input v-model.number="query.id" :placeholder="$t('storage_channels.id')" clearable class="w-32" />
          <el-select v-model="query.provider" :placeholder="$t('storage_channels.provider')" clearable class="w-44">
            <el-option :label="$t('storage_channels.provider_aliyun_oss')" value="aliyun_oss" />
            <el-option :label="$t('storage_channels.provider_cloudflare_r2')" value="cloudflare_r2" />
          </el-select>
          <el-select v-model="query.status" :placeholder="$t('storage_channels.status')" clearable class="w-32">
            <el-option :label="$t('storage_channels.status_1')" :value="StorageChannelStatus.Enabled" />
            <el-option :label="$t('storage_channels.status_0')" :value="StorageChannelStatus.Disabled" />
          </el-select>
          <el-button type="primary" @click="reload">{{ $t('common.search') }}</el-button>
          <el-button @click="reset">{{ $t('common.reset') }}</el-button>
          <el-button type="success" @click="openCreate">{{ $t('common.create') }}</el-button>
        </div>
      </div>
    </el-card>

    <el-card shadow="never">
      <el-table v-loading="loading" :data="rows" stripe size="large" style="width: 100%">
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column :label="$t('storage_channels.channel')" min-width="220">
          <template #default="{ row }">
            <div class="flex flex-col leading-tight">
              <span class="font-medium">{{ channelName(row) }}</span>
              <span class="text-xs text-gray-500">{{ row.config.public_base_url || '-' }}</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column :label="$t('storage_channels.provider')" width="150">
          <template #default="{ row }">
            <el-tag :type="row.provider === 'aliyun_oss' ? 'warning' : 'success'">{{ providerLabel(row.provider) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="$t('storage_channels.bucket')" min-width="150">
          <template #default="{ row }">{{ row.config.bucket || '-' }}</template>
        </el-table-column>
        <el-table-column :label="$t('storage_channels.endpoint')" min-width="220">
          <template #default="{ row }">{{ row.config.endpoint || '-' }}</template>
        </el-table-column>
        <el-table-column :label="$t('storage_channels.prefix')" min-width="130">
          <template #default="{ row }">{{ row.config.prefix || '-' }}</template>
        </el-table-column>
        <el-table-column :label="$t('storage_channels.sort_order')" width="110">
          <template #default="{ row }">{{ row.sort_order }}</template>
        </el-table-column>
        <el-table-column :label="$t('storage_channels.status')" width="110">
          <template #default="{ row }">
            <el-tag :type="row.status === StorageChannelStatus.Enabled ? 'success' : 'info'">
              {{ row.status === StorageChannelStatus.Enabled ? $t('storage_channels.status_1') : $t('storage_channels.status_0') }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="$t('orders.updated')" min-width="180">
          <template #default="{ row }">{{ row.updated_at }}</template>
        </el-table-column>
        <el-table-column :label="$t('common.actions')" width="150" fixed="right" align="right">
          <template #default="{ row }">
            <el-button size="small" type="primary" plain @click="openEdit(row)">{{ $t('common.edit') }}</el-button>
            <el-button size="small" type="danger" plain @click="remove(row)">{{ $t('common.delete') }}</el-button>
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

    <el-dialog v-model="dialog.visible" :title="dialog.mode === 'create' ? $t('common.create') : $t('common.edit')" width="840px">
      <el-form ref="formRef" :model="form" :rules="rules" label-width="150px" class="pr-4">
        <div class="grid grid-cols-2 gap-x-4">
          <el-form-item :label="$t('storage_channels.name')" prop="name">
            <el-input v-model="form.name" />
          </el-form-item>
          <el-form-item :label="$t('storage_channels.provider')" prop="provider">
            <el-select v-model="form.provider" class="w-full" @change="onProviderChange">
              <el-option :label="$t('storage_channels.provider_aliyun_oss')" value="aliyun_oss" />
              <el-option :label="$t('storage_channels.provider_cloudflare_r2')" value="cloudflare_r2" />
            </el-select>
          </el-form-item>
          <el-form-item :label="$t('storage_channels.status')">
            <el-switch v-model="form.enabled" />
          </el-form-item>
          <el-form-item :label="$t('storage_channels.sort_order')" prop="sort_order">
            <el-input-number v-model="form.sort_order" :min="0" class="w-full" />
          </el-form-item>
        </div>

        <el-divider content-position="left">{{ $t('storage_channels.config') }}</el-divider>

        <div class="grid grid-cols-2 gap-x-4">
          <el-form-item :label="$t('storage_channels.bucket')" prop="config.bucket">
            <el-input v-model="form.config.bucket" />
          </el-form-item>
          <el-form-item v-if="form.provider === 'cloudflare_r2'" :label="$t('storage_channels.region')" prop="config.region">
            <el-input v-model="form.config.region" placeholder="auto" />
          </el-form-item>
          <el-form-item :label="$t('storage_channels.endpoint')" prop="config.endpoint">
            <el-input v-model="form.config.endpoint" :placeholder="endpointPlaceholder" />
          </el-form-item>
          <el-form-item v-if="form.provider === 'cloudflare_r2'" :label="$t('storage_channels.public_base_url')" prop="config.public_base_url">
            <el-input v-model="form.config.public_base_url" :placeholder="publicBaseUrlPlaceholder" />
          </el-form-item>
          <el-form-item v-if="form.provider === 'aliyun_oss'" :label="$t('storage_channels.storage_class')" prop="config.storage_class">
            <el-select v-model="form.config.storage_class" class="w-full">
              <el-option :label="$t('storage_channels.storage_class_standard')" value="standard" />
              <el-option :label="$t('storage_channels.storage_class_ia')" value="ia" />
              <el-option :label="$t('storage_channels.storage_class_archive')" value="archive" />
              <el-option :label="$t('storage_channels.storage_class_cold_archive')" value="cold_archive" />
              <el-option :label="$t('storage_channels.storage_class_deep_cold_archive')" value="deep_cold_archive" />
            </el-select>
          </el-form-item>
          <el-form-item v-if="form.provider === 'aliyun_oss'" :label="$t('storage_channels.object_acl')" prop="config.object_acl">
            <el-select v-model="form.config.object_acl" class="w-full">
              <el-option :label="$t('storage_channels.object_acl_public_read')" value="public-read" />
              <el-option :label="$t('storage_channels.object_acl_default')" value="default" />
              <el-option :label="$t('storage_channels.object_acl_private')" value="private" />
              <el-option :label="$t('storage_channels.object_acl_public_read_write')" value="public-read-write" />
            </el-select>
          </el-form-item>
          <el-form-item :label="$t('storage_channels.access_key_id')" prop="config.access_key_id">
            <el-input v-model="form.config.access_key_id" />
          </el-form-item>
          <el-form-item :label="$t('storage_channels.access_key_secret')" prop="config.access_key_secret">
            <el-input v-model="form.config.access_key_secret" type="password" show-password />
          </el-form-item>
          <el-form-item :label="$t('storage_channels.prefix')" prop="config.prefix" class="col-span-2">
            <el-input v-model="form.config.prefix" :placeholder="$t('storage_channels.prefix_placeholder')" />
          </el-form-item>
        </div>

        <div class="rounded border border-slate-200 bg-slate-50 px-3 py-2 text-xs leading-5 text-slate-500">
          {{ $t(providerHelpKey) }}
        </div>
      </el-form>
      <template #footer>
        <el-button @click="dialog.visible = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submit">{{ $t('common.save') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox, type FormInstance, type FormRules } from 'element-plus'
import {
  createStorageChannel,
  deleteStorageChannel,
  fetchStorageChannels,
  updateStorageChannel,
} from '@/apis/storage'
import {
  StorageChannelStatus,
  type ListStorageChannelsParams,
  type SaveStorageChannelReq,
  type StorageChannel,
  type StorageChannelConfig,
  type StorageProvider,
} from '@/types/storage'

type StorageChannelForm = {
  name: string
  provider: StorageProvider
  enabled: boolean
  sort_order: number
  config: StorageChannelConfig
}

const { t } = useI18n()

const rows = ref<StorageChannel[]>([])
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const loading = ref(false)
const query = reactive<ListStorageChannelsParams>({ id: undefined, provider: undefined, status: undefined })
const dialog = reactive({ visible: false, mode: 'create' as 'create' | 'edit', editingId: undefined as number | undefined })
const formRef = ref<FormInstance>()

const form = reactive<StorageChannelForm>({
  name: '',
  provider: 'aliyun_oss',
  enabled: true,
  sort_order: 0,
  config: defaultConfig('aliyun_oss'),
})

const rules = reactive<FormRules<StorageChannelForm>>({
  name: [{ required: true, message: t('storage_channels.name_required'), trigger: 'blur' }],
  provider: [{ required: true, message: t('storage_channels.provider_required'), trigger: 'change' }],
  'config.bucket': [{ required: true, message: t('storage_channels.bucket_required'), trigger: 'blur' }],
  'config.endpoint': [{ required: true, message: t('storage_channels.endpoint_required'), trigger: 'blur' }],
  'config.access_key_id': [{ required: true, message: t('storage_channels.access_key_id_required'), trigger: 'blur' }],
  'config.access_key_secret': [{ required: true, message: t('storage_channels.access_key_secret_required'), trigger: 'blur' }],
  'config.public_base_url': [
    {
      validator: (_rule, value, callback) => {
        if (form.provider === 'cloudflare_r2' && !String(value || '').trim()) {
          callback(new Error(t('storage_channels.public_base_url_required') as string))
          return
        }
        callback()
      },
      trigger: 'blur',
    },
  ],
})

const providerHelpKey = computed(() => form.provider === 'aliyun_oss' ? 'storage_channels.help_aliyun_oss' : 'storage_channels.help_cloudflare_r2')
const endpointPlaceholder = computed(() => form.provider === 'aliyun_oss' ? 'oss-cn-guangzhou.aliyuncs.com' : 'https://<account_id>.r2.cloudflarestorage.com')
const publicBaseUrlPlaceholder = computed(() => 'https://apphub.1postpro.com/apphub 或 https://pub-xxxx.r2.dev')

function defaultConfig(provider: StorageProvider): StorageChannelConfig {
  return {
    bucket: '',
    region: provider === 'cloudflare_r2' ? 'auto' : '',
    endpoint: '',
    access_key_id: '',
    access_key_secret: '',
    public_base_url: '',
    prefix: 'apps',
    storage_class: provider === 'aliyun_oss' ? 'standard' : undefined,
    object_acl: provider === 'aliyun_oss' ? 'public-read' : undefined,
  }
}

function providerLabel(provider: StorageProvider) {
  return provider === 'aliyun_oss' ? t('storage_channels.provider_aliyun_oss') : t('storage_channels.provider_cloudflare_r2')
}

function channelName(row: StorageChannel) {
  if (row.name) return row.name
  const bucket = row.config?.bucket || '-'
  const prefix = row.config?.prefix ? `/${row.config.prefix}` : ''
  return `${providerLabel(row.provider)} - ${bucket}${prefix}`
}

function assignForm(next: StorageChannelForm) {
  form.name = next.name
  form.provider = next.provider
  form.enabled = next.enabled
  form.sort_order = next.sort_order
  form.config = { ...next.config }
}

function onProviderChange(provider: StorageProvider) {
  assignForm({
    name: form.name,
    provider,
    enabled: form.enabled,
    sort_order: form.sort_order,
    config: { ...defaultConfig(provider), bucket: form.config.bucket, prefix: form.config.prefix },
  })
  formRef.value?.clearValidate()
}

async function reload() {
  loading.value = true
  try {
    const data = await fetchStorageChannels({ ...query, page: page.value, page_size: pageSize.value })
    rows.value = data.list
    total.value = data.total
  } finally {
    loading.value = false
  }
}

function reset() {
  query.id = undefined
  query.provider = undefined
  query.status = undefined
  page.value = 1
  reload()
}

function handlePageChange(p: number) {
  page.value = p
  reload()
}

function handleSizeChange(s: number) {
  pageSize.value = s
  page.value = 1
  reload()
}

function openCreate() {
  dialog.mode = 'create'
  dialog.editingId = undefined
  assignForm({
    name: '',
    provider: 'aliyun_oss',
    enabled: true,
    sort_order: 0,
    config: defaultConfig('aliyun_oss'),
  })
  dialog.visible = true
  formRef.value?.clearValidate()
}

function openEdit(row: StorageChannel) {
  dialog.mode = 'edit'
  dialog.editingId = row.id
  assignForm({
    name: row.name || channelName(row),
    provider: row.provider,
    enabled: row.status === StorageChannelStatus.Enabled,
    sort_order: row.sort_order,
    config: { ...defaultConfig(row.provider), ...(row.config || {}) },
  })
  dialog.visible = true
  formRef.value?.clearValidate()
}

function trimConfig(config: StorageChannelConfig): StorageChannelConfig {
  const endpoint = config.endpoint.trim()
  const publicBaseUrl = form.provider === 'aliyun_oss' ? '' : config.public_base_url.trim()
  return {
    bucket: config.bucket.trim(),
    region: form.provider === 'cloudflare_r2' ? config.region?.trim() || undefined : undefined,
    endpoint,
    access_key_id: config.access_key_id.trim(),
    access_key_secret: config.access_key_secret.trim(),
    public_base_url: publicBaseUrl,
    prefix: config.prefix.trim(),
    storage_class: form.provider === 'aliyun_oss' ? config.storage_class || 'standard' : undefined,
    object_acl: form.provider === 'aliyun_oss' ? config.object_acl || 'public-read' : undefined,
  }
}

function buildPayload(): SaveStorageChannelReq {
  return {
    name: form.name.trim(),
    provider: form.provider,
    status: form.enabled ? StorageChannelStatus.Enabled : StorageChannelStatus.Disabled,
    sort_order: form.sort_order,
    config: trimConfig(form.config),
  }
}

async function submit() {
  const valid = await formRef.value?.validate()
  if (!valid) {
    ElMessage.warning(t('common.please_check_form') as string)
    return
  }

  const payload = buildPayload()
  if (dialog.mode === 'edit' && dialog.editingId != null) await updateStorageChannel(dialog.editingId, payload)
  else await createStorageChannel(payload)

  dialog.visible = false
  ElMessage.success(t(dialog.mode === 'edit' ? 'common.saved' : 'common.created') as string)
  reload()
}

async function remove(row: StorageChannel) {
  await ElMessageBox.confirm(t('storage_channels.delete_confirm', { name: channelName(row) }) as string, t('common.confirm') as string, { type: 'warning' })
  await deleteStorageChannel(row.id)
  ElMessage.success(t('common.deleted') as string)
  reload()
}

onMounted(reload)
</script>
