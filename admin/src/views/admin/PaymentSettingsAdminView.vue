<template>
  <div class="space-y-4">
    <el-card shadow="hover">
      <div class="flex items-center justify-between gap-3">
        <h2 class="text-xl font-semibold">{{ $t('payment_settings.title') }}</h2>
        <div class="flex items-center gap-2">
          <el-select v-model="query.provider" :placeholder="$t('payment_settings.provider')" clearable class="w-36">
            <el-option :label="$t('payment_settings.provider_wechat')" value="wechat" />
            <el-option :label="$t('payment_settings.provider_alipay')" value="alipay" />
          </el-select>
          <el-select v-model="query.status" :placeholder="$t('payment_settings.status')" clearable class="w-32">
            <el-option :label="$t('payment_settings.status_1')" :value="PaymentChannelStatus.Enabled" />
            <el-option :label="$t('payment_settings.status_0')" :value="PaymentChannelStatus.Disabled" />
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
        <el-table-column prop="name" :label="$t('payment_settings.name')" min-width="170" />
        <el-table-column :label="$t('payment_settings.provider')" width="120">
          <template #default="{ row }">
            <el-tag :type="row.provider === 'wechat' ? 'success' : 'warning'">{{ providerLabel(row.provider) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="pay_type" :label="$t('payment_settings.pay_type')" min-width="130" />
        <el-table-column prop="sort_order" :label="$t('payment_settings.sort_order')" width="100" />
        <el-table-column :label="$t('payment_settings.status')" width="110">
          <template #default="{ row }">
            <el-tag :type="row.status === PaymentChannelStatus.Enabled ? 'success' : 'info'">
              {{ row.status === PaymentChannelStatus.Enabled ? $t('payment_settings.status_1') : $t('payment_settings.status_0') }}
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

    <el-dialog v-model="dialog.visible" :title="dialog.mode === 'create' ? $t('common.create') : $t('common.edit')" width="760px">
      <el-form ref="formRef" :model="form" :rules="rules" label-width="150px" class="pr-4">
        <div class="grid grid-cols-2 gap-x-4">
          <el-form-item :label="$t('payment_settings.name')" prop="name">
            <el-input v-model="form.name" />
          </el-form-item>
          <el-form-item :label="$t('payment_settings.provider')" prop="provider">
            <el-select v-model="form.provider" class="w-full" @change="onProviderChange">
              <el-option :label="$t('payment_settings.provider_wechat')" value="wechat" />
              <el-option :label="$t('payment_settings.provider_alipay')" value="alipay" />
            </el-select>
          </el-form-item>
          <el-form-item :label="$t('payment_settings.pay_type')" prop="pay_type">
            <el-input v-model="form.pay_type" />
          </el-form-item>
          <el-form-item :label="$t('payment_settings.sort_order')">
            <el-input-number v-model.number="form.sort_order" :min="0" class="w-full" />
          </el-form-item>
          <el-form-item :label="$t('payment_settings.status')">
            <el-switch v-model="form.enabled" />
          </el-form-item>
        </div>

        <el-divider content-position="left">{{ $t('payment_settings.config') }}</el-divider>

        <div class="grid grid-cols-2 gap-x-4">
          <el-form-item
            v-for="field in activeTextFields"
            :key="field.key"
            :label="$t(field.label)"
            :required="field.required"
          >
            <el-input v-model="form.config[field.key]" />
          </el-form-item>
        </div>

        <el-form-item
          v-for="field in activeSecretFields"
          :key="field.key"
          :label="$t(field.label)"
          :required="field.required"
        >
          <div class="w-full space-y-2">
            <div class="flex items-center gap-2">
              <el-button size="small" @click="selectFile(field.key)">{{ $t('common.upload') }}</el-button>
              <span class="truncate text-xs text-gray-500">{{ uploadNames[field.key] || $t('payment_settings.upload_hint') }}</span>
            </div>
            <el-input v-model="form.config[field.key]" type="textarea" :rows="4" resize="vertical" />
          </div>
        </el-form-item>
      </el-form>
      <input
        ref="fileInputRef"
        style="display: none"
        type="file"
        accept=".pem,.key,.crt,.cer,.txt"
        @change="onFileSelected"
      />
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
  createPaymentChannel,
  deletePaymentChannel,
  fetchPaymentChannels,
  updatePaymentChannel,
} from '@/apis/payments'
import {
  PaymentChannelStatus,
  type ListPaymentChannelsParams,
  type PaymentChannel,
  type PaymentChannelConfig,
  type PaymentProvider,
  type SavePaymentChannelReq,
} from '@/types/payments'

type ConfigKey = keyof PaymentChannelConfig

type ConfigField = {
  key: ConfigKey
  label: string
  required: boolean
  secret?: boolean
}

type PaymentChannelForm = {
  name: string
  provider: PaymentProvider
  pay_type: string
  enabled: boolean
  sort_order: number
  config: PaymentChannelConfig
}

const { t } = useI18n()

const rows = ref<PaymentChannel[]>([])
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const loading = ref(false)
const query = reactive<ListPaymentChannelsParams>({ provider: undefined, status: undefined })
const dialog = reactive({ visible: false, mode: 'create' as 'create' | 'edit', editingId: undefined as number | undefined })
const formRef = ref<FormInstance>()
const fileInputRef = ref<HTMLInputElement>()
const pendingFileField = ref<ConfigKey>()
const uploadNames = reactive<Partial<Record<ConfigKey, string>>>({})

const form = reactive<PaymentChannelForm>({
  name: '',
  provider: 'wechat',
  pay_type: 'wechat_native',
  enabled: true,
  sort_order: 0,
  config: defaultConfig('wechat'),
})

const rules = reactive<FormRules<PaymentChannelForm>>({
  name: [{ required: true, message: 'Name required', trigger: 'blur' }],
  provider: [{ required: true, message: 'Provider required', trigger: 'change' }],
  pay_type: [{ required: true, message: 'Pay type required', trigger: 'blur' }],
})

const configFields: Record<PaymentProvider, ConfigField[]> = {
  wechat: [
    { key: 'app_id', label: 'payment_settings.app_id', required: true },
    { key: 'mch_id', label: 'payment_settings.mch_id', required: true },
    { key: 'merchant_serial_no', label: 'payment_settings.merchant_serial_no', required: true },
    { key: 'api_base_url', label: 'payment_settings.api_base_url', required: false },
    { key: 'api_v3_key', label: 'payment_settings.api_v3_key', required: true, secret: true },
    { key: 'merchant_private_key_pem', label: 'payment_settings.merchant_private_key_pem', required: true, secret: true },
    { key: 'platform_public_key_pem', label: 'payment_settings.platform_public_key_pem', required: true, secret: true },
  ],
  alipay: [
    { key: 'app_id', label: 'payment_settings.app_id', required: true },
    { key: 'gateway_url', label: 'payment_settings.gateway_url', required: false },
    { key: 'seller_id', label: 'payment_settings.seller_id', required: false },
    { key: 'app_private_key_pem', label: 'payment_settings.app_private_key_pem', required: true, secret: true },
    { key: 'alipay_public_key_pem', label: 'payment_settings.alipay_public_key_pem', required: true, secret: true },
  ],
}

const activeFields = computed(() => configFields[form.provider])
const activeTextFields = computed(() => activeFields.value.filter(field => !field.secret))
const activeSecretFields = computed(() => activeFields.value.filter(field => field.secret))

function defaultConfig(provider: PaymentProvider): PaymentChannelConfig {
  if (provider === 'wechat') {
    return {
      app_id: '',
      mch_id: '',
      merchant_serial_no: '',
      merchant_private_key_pem: '',
      api_v3_key: '',
      platform_public_key_pem: '',
      api_base_url: 'https://api.mch.weixin.qq.com',
    }
  }

  return {
    app_id: '',
    app_private_key_pem: '',
    alipay_public_key_pem: '',
    gateway_url: 'https://openapi.alipay.com/gateway.do',
    seller_id: '',
  }
}

function providerLabel(provider: PaymentProvider) {
  return provider === 'wechat' ? t('payment_settings.provider_wechat') : t('payment_settings.provider_alipay')
}

function resetUploadNames() {
  for (const key of Object.keys(uploadNames) as ConfigKey[]) delete uploadNames[key]
}

function assignForm(next: PaymentChannelForm) {
  form.name = next.name
  form.provider = next.provider
  form.pay_type = next.pay_type
  form.enabled = next.enabled
  form.sort_order = next.sort_order
  form.config = { ...next.config }
}

function onProviderChange(provider: PaymentProvider) {
  assignForm({
    ...form,
    provider,
    pay_type: provider === 'wechat' ? 'wechat_native' : 'alipay_page',
    config: defaultConfig(provider),
  })
  resetUploadNames()
  formRef.value?.clearValidate()
}

async function reload() {
  loading.value = true
  try {
    const data = await fetchPaymentChannels({ ...query, page: page.value, page_size: pageSize.value })
    rows.value = data.list
    total.value = data.total
  } finally {
    loading.value = false
  }
}

function reset() {
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
    provider: 'wechat',
    pay_type: 'wechat_native',
    enabled: true,
    sort_order: 0,
    config: defaultConfig('wechat'),
  })
  resetUploadNames()
  dialog.visible = true
  formRef.value?.clearValidate()
}

function openEdit(row: PaymentChannel) {
  dialog.mode = 'edit'
  dialog.editingId = row.id
  assignForm({
    name: row.name,
    provider: row.provider,
    pay_type: row.pay_type,
    enabled: row.status === PaymentChannelStatus.Enabled,
    sort_order: row.sort_order,
    config: { ...defaultConfig(row.provider), ...(row.config || {}) },
  })
  resetUploadNames()
  dialog.visible = true
  formRef.value?.clearValidate()
}

function selectFile(field: ConfigKey) {
  pendingFileField.value = field
  if (fileInputRef.value) {
    fileInputRef.value.value = ''
    fileInputRef.value.click()
  }
}

function onFileSelected(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  const field = pendingFileField.value
  if (!file || !field) return

  const reader = new FileReader()
  reader.onload = () => {
    form.config[field] = String(reader.result || '')
    uploadNames[field] = file.name
    ElMessage.success(t('payment_settings.file_loaded') as string)
  }
  reader.onerror = () => ElMessage.error(t('payment_settings.file_load_failed') as string)
  reader.readAsText(file)
}

function validateConfig() {
  const missing = activeFields.value.find(field => field.required && !String(form.config[field.key] || '').trim())
  if (missing) {
    ElMessage.warning(t('payment_settings.required_config', { field: t(missing.label) }) as string)
    return false
  }
  return true
}

function buildPayload(): SavePaymentChannelReq {
  const allowedKeys = activeFields.value.map(field => field.key)
  const config = allowedKeys.reduce<PaymentChannelConfig>((next, key) => {
    const value = form.config[key]
    if (value != null) next[key] = String(value)
    return next
  }, {})

  return {
    name: form.name.trim(),
    provider: form.provider,
    pay_type: form.pay_type.trim(),
    status: form.enabled ? PaymentChannelStatus.Enabled : PaymentChannelStatus.Disabled,
    sort_order: form.sort_order,
    config,
  }
}

async function submit() {
  const valid = await formRef.value?.validate()
  if (!valid || !validateConfig()) {
    ElMessage.warning(t('common.please_check_form') as string)
    return
  }

  const payload = buildPayload()
  if (dialog.mode === 'edit' && dialog.editingId != null) await updatePaymentChannel(dialog.editingId, payload)
  else await createPaymentChannel(payload)

  dialog.visible = false
  ElMessage.success(t(dialog.mode === 'edit' ? 'common.saved' : 'common.created') as string)
  reload()
}

async function remove(row: PaymentChannel) {
  await ElMessageBox.confirm(t('payment_settings.delete_confirm', { name: row.name }) as string, t('common.confirm') as string, { type: 'warning' })
  await deletePaymentChannel(row.id)
  ElMessage.success(t('common.deleted') as string)
  reload()
}

onMounted(reload)
</script>
