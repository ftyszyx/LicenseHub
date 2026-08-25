<template>
  <div class="space-y-4">
    <el-card shadow="hover">
      <div class="flex items-center justify-between gap-3">
        <h2 class="text-xl font-semibold">{{ $t('system_settings.title') }}</h2>
        <el-button type="primary" :loading="saving" @click="submit">{{ $t('common.save') }}</el-button>
      </div>
    </el-card>

    <el-card shadow="never">
      <template #header><h3 class="text-base font-semibold">注册与邮件验证</h3></template>
      <el-form v-loading="loading" :model="form" label-width="170px" class="max-w-3xl">
        <el-form-item label="开放用户注册">
          <el-switch v-model="form.registration_enabled" />
          <span class="ml-3 text-sm text-gray-500">关闭后隐藏注册入口并停用验证码接口</span>
        </el-form-item>
        <el-form-item label="邮件发送模式">
          <el-segmented v-model="form.email_service_mode" :options="emailModeOptions" />
        </el-form-item>
        <el-form-item label="发件人">
          <el-input v-model="form.email_from" placeholder="LicenseHub <no-reply@example.com>" />
        </el-form-item>
        <template v-if="form.email_service_mode === 'smtp'">
          <el-form-item label="SMTP 服务器">
            <el-input v-model="form.email_smtp_host" placeholder="smtp.example.com" />
          </el-form-item>
          <el-form-item label="SMTP 端口">
            <el-input-number v-model="form.email_smtp_port" :min="1" :max="65535" />
          </el-form-item>
          <el-form-item label="TLS 模式">
            <el-select v-model="form.email_smtp_tls_mode" class="w-48">
              <el-option label="STARTTLS" value="starttls" />
              <el-option label="直接 TLS" value="tls" />
              <el-option label="不加密" value="none" />
            </el-select>
          </el-form-item>
          <el-form-item label="SMTP 用户名">
            <el-input v-model="form.email_smtp_username" autocomplete="off" />
          </el-form-item>
          <el-form-item label="SMTP 密码">
            <el-input
              v-model="form.email_smtp_password"
              type="password"
              show-password
              autocomplete="new-password"
              :placeholder="emailPasswordSet ? '已设置，留空保持不变' : '请输入 SMTP 密码'"
            />
          </el-form-item>
        </template>
        <el-form-item label="测试收件邮箱">
          <div class="flex w-full flex-wrap gap-2">
            <el-input v-model="testEmail" class="max-w-md" placeholder="test@example.com" />
            <el-button :loading="testingEmail" @click="sendTestEmail">发送测试邮件</el-button>
          </div>
          <p v-if="form.email_service_mode === 'log'" class="mt-1 text-xs text-gray-500">日志模式不会真实发信，测试验证码会写入服务端开发日志。</p>
        </el-form-item>
      </el-form>
    </el-card>

    <el-card shadow="never">
      <template #header><h3 class="text-base font-semibold">通用资源设置</h3></template>
      <el-form v-loading="loading" :model="form" label-width="170px" class="max-w-3xl">
        <el-form-item label="资源存储渠道">
          <el-select v-model="form.resource_storage_channel_id" class="w-80">
            <el-option label="自动选择（按存储渠道排序）" :value="0" />
            <el-option
              v-for="channel in storageChannels"
              :key="channel.id"
              :label="storageChannelLabel(channel)"
              :value="channel.id"
              :disabled="channel.status !== StorageChannelStatus.Enabled"
            />
          </el-select>
          <span class="ml-3 text-sm text-gray-500">以后所有图片和文件资源默认使用此渠道</span>
        </el-form-item>
      </el-form>
    </el-card>

    <el-card shadow="never">
      <template #header><h3 class="text-base font-semibold">分销设置</h3></template>
      <el-form v-loading="loading" :model="form" label-width="170px" class="max-w-3xl">
        <el-form-item label="开放分销功能">
          <el-switch v-model="form.distribution_enabled" />
        </el-form-item>
        <el-form-item label="永久邀请绑定">
          <el-switch
            v-model="form.distribution_referrer_binding_enabled"
            :disabled="!form.distribution_enabled"
          />
          <span class="ml-2 text-sm text-gray-500">仅影响新用户注册，关闭不会解除已有邀请关系</span>
        </el-form-item>
        <el-form-item label="默认佣金比例">
          <el-input-number v-model="form.distribution_default_rate_percent" :min="0" :max="100" :precision="2" />
          <span class="ml-2 text-sm text-gray-500">%</span>
        </el-form-item>
        <el-form-item label="推广归因有效期">
          <el-input-number v-model="form.distribution_attribution_days" :min="1" :max="3650" />
          <span class="ml-2 text-sm text-gray-500">天，点击推广链接后在此期限内下单归属推广用户</span>
        </el-form-item>
        <el-form-item label="佣金观察期">
          <el-input-number v-model="form.distribution_holding_days" :min="0" :max="3650" />
          <span class="ml-2 text-sm text-gray-500">天，支付成功后经过此期限佣金变为可结算</span>
        </el-form-item>
        <el-form-item label="最低提现金额">
          <el-input-number v-model="form.distribution_min_withdraw_yuan" :min="0" :precision="2" />
          <span class="ml-2 text-sm text-gray-500">元，提现功能暂未开放</span>
        </el-form-item>
      </el-form>
    </el-card>

    <el-card shadow="never">
      <el-form
        ref="formRef"
        v-loading="loading"
        :model="form"
        :rules="rules"
        label-width="150px"
        class="max-w-3xl"
      >
        <el-form-item :label="$t('system_settings.storefront_title')" prop="storefront_title">
          <el-input
            v-model="form.storefront_title"
            maxlength="80"
            show-word-limit
            :placeholder="$t('system_settings.storefront_title_placeholder')"
          />
          <div class="mt-1 text-xs leading-5 text-gray-500">
            {{ $t('system_settings.storefront_title_help') }}
          </div>
        </el-form-item>
      </el-form>
    </el-card>

    <el-card shadow="never">
      <div class="mb-4 flex items-center justify-between gap-3">
        <div>
          <h3 class="text-base font-semibold">{{ $t('system_settings.license_signing_title') }}</h3>
          <p class="mt-1 text-xs leading-5 text-gray-500">
            {{ $t('system_settings.license_signing_help') }}
          </p>
        </div>
        <el-tag :type="licenseSigning.configured ? 'success' : 'warning'">
          {{ licenseSigning.configured ? $t('system_settings.license_key_configured') : $t('system_settings.license_key_missing') }}
        </el-tag>
      </div>

      <el-form label-width="150px" class="max-w-3xl">
        <el-form-item :label="$t('system_settings.license_key_id')">
          <el-input :model-value="licenseSigning.key_id || 'license-v1'" readonly />
        </el-form-item>
        <el-form-item :label="$t('system_settings.license_public_key')">
          <el-input
            :model-value="licenseSigning.public_key_b64 || ''"
            readonly
            :placeholder="$t('system_settings.license_public_key_placeholder')"
          >
            <template #append>
              <el-button :disabled="!licenseSigning.public_key_b64" @click="copyPublicKey">
                {{ $t('common.copy') }}
              </el-button>
            </template>
          </el-input>
          <!-- <div class="mt-1 text-xs leading-5 text-gray-500">
            {{ $t('system_settings.license_public_key_help') }}
          </div> -->
        </el-form-item>
        <el-form-item :label="$t('system_settings.license_private_key')">
          <el-input
            :model-value="licenseSigning.private_key_b64 || ''"
            readonly
            show-password
            :placeholder="$t('system_settings.license_private_key_placeholder')"
          >
            <template #append>
              <el-button :disabled="!licenseSigning.private_key_b64" @click="copyPrivateKey">
                {{ $t('common.copy') }}
              </el-button>
            </template>
          </el-input>
          <div class="mt-1 text-xs leading-5 text-red-500">
            {{ $t('system_settings.license_private_key_help') }}
          </div>
        </el-form-item>
        <el-form-item :label="$t('system_settings.license_updated_at')">
          <el-input :model-value="licenseSigning.updated_at || '-'" readonly />
        </el-form-item>
        <el-form-item>
          <div class="flex flex-wrap gap-2">
            <el-button
              type="primary"
              :loading="generating"
              :disabled="licenseSigning.configured"
              @click="generateKey(false)"
            >
              {{ $t('system_settings.generate_license_key') }}
            </el-button>
            <el-button
              type="danger"
              plain
              :loading="generating"
              :disabled="!licenseSigning.configured"
              @click="confirmRotate"
            >
              {{ $t('system_settings.rotate_license_key') }}
            </el-button>
          </div>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox, type FormInstance, type FormRules } from 'element-plus'
import { fetchSystemSettings, generateLicenseSigningKey, sendSystemTestEmail, updateSystemSettings } from '@/apis/system_settings'
import { fetchStorageChannels } from '@/apis/storage'
import type { LicenseSigningInfo, SaveSystemSettingsReq, SiteSettings } from '@/types'
import { StorageChannelStatus, type StorageChannel } from '@/types/storage'

const { t } = useI18n()
const formRef = ref<FormInstance>()
const loading = ref(false)
const saving = ref(false)
const generating = ref(false)
const testingEmail = ref(false)
const testEmail = ref('')
const emailPasswordSet = ref(false)
const emailModeOptions = [
  { label: '开发日志', value: 'log' },
  { label: 'SMTP', value: 'smtp' },
]
const storageChannels = ref<StorageChannel[]>([])

type SettingsForm = SaveSystemSettingsReq & {
  distribution_default_rate_percent: number
  distribution_min_withdraw_yuan: number
}

const form = reactive<SettingsForm>({
  storefront_title: 'LicenseHub',
  registration_enabled: false,
  resource_storage_channel_id: 0,
  distribution_enabled: false,
  distribution_referrer_binding_enabled: false,
  distribution_default_rate_bps: 2000,
  distribution_attribution_days: 30,
  distribution_holding_days: 7,
  distribution_min_withdraw_cents: 5000,
  distribution_default_rate_percent: 20,
  distribution_min_withdraw_yuan: 50,
  email_service_mode: 'log',
  email_from: 'LicenseHub <no-reply@example.com>',
  email_smtp_host: '',
  email_smtp_port: 587,
  email_smtp_username: '',
  email_smtp_password: '',
  email_smtp_tls_mode: 'starttls',
})

const licenseSigning = reactive<LicenseSigningInfo>({
  configured: false,
  key_id: 'license-v1',
  public_key_b64: null,
  private_key_b64: null,
  updated_at: null,
})

const rules = reactive<FormRules<SaveSystemSettingsReq>>({
  storefront_title: [
    { required: true, message: 'Storefront title required', trigger: 'blur' },
    { max: 80, message: 'Storefront title is too long', trigger: 'blur' },
  ],
})

async function load() {
  loading.value = true
  try {
    const data = await fetchSystemSettings()
    try {
      const channels = await fetchStorageChannels({ page: 1, page_size: 100 })
      storageChannels.value = channels.list
    } catch {
      storageChannels.value = []
    }
    assignSettings(data)
  } finally {
    loading.value = false
  }
}

function assignSettings(data: SiteSettings) {
  form.storefront_title = data.storefront_title || 'LicenseHub'
  form.registration_enabled = Boolean(data.registration_enabled)
  form.resource_storage_channel_id = data.resource_storage_channel_id ?? 0
  form.distribution_enabled = Boolean(data.distribution?.enabled)
  form.distribution_referrer_binding_enabled = Boolean(data.distribution?.referrer_binding_enabled)
  form.distribution_default_rate_bps = data.distribution?.default_rate_bps ?? 2000
  form.distribution_attribution_days = data.distribution?.attribution_days ?? 30
  form.distribution_holding_days = data.distribution?.holding_days ?? 7
  form.distribution_min_withdraw_cents = data.distribution?.min_withdraw_cents ?? 5000
  form.distribution_default_rate_percent = form.distribution_default_rate_bps / 100
  form.distribution_min_withdraw_yuan = form.distribution_min_withdraw_cents / 100
  form.email_service_mode = data.email?.mode || 'log'
  form.email_from = data.email?.from || 'LicenseHub <no-reply@example.com>'
  form.email_smtp_host = data.email?.smtp_host || ''
  form.email_smtp_port = data.email?.smtp_port || 587
  form.email_smtp_username = data.email?.smtp_username || ''
  form.email_smtp_password = ''
  form.email_smtp_tls_mode = data.email?.smtp_tls_mode || 'starttls'
  emailPasswordSet.value = Boolean(data.email?.smtp_password_set)
  licenseSigning.configured = Boolean(data.license_signing?.configured)
  licenseSigning.key_id = data.license_signing?.key_id || 'license-v1'
  licenseSigning.public_key_b64 = data.license_signing?.public_key_b64 || null
  licenseSigning.private_key_b64 = data.license_signing?.private_key_b64 || null
  licenseSigning.updated_at = data.license_signing?.updated_at || null
}

async function submit() {
  const valid = await formRef.value?.validate()
  if (!valid) {
    ElMessage.warning(t('common.please_check_form') as string)
    return
  }

  saving.value = true
  try {
    const data = await updateSystemSettings({
      storefront_title: form.storefront_title.trim(),
      registration_enabled: form.registration_enabled,
      resource_storage_channel_id: form.resource_storage_channel_id,
      distribution_enabled: form.distribution_enabled,
      distribution_referrer_binding_enabled: form.distribution_referrer_binding_enabled,
      distribution_default_rate_bps: Math.round(form.distribution_default_rate_percent * 100),
      distribution_attribution_days: form.distribution_attribution_days,
      distribution_holding_days: form.distribution_holding_days,
      distribution_min_withdraw_cents: Math.round(form.distribution_min_withdraw_yuan * 100),
      email_service_mode: form.email_service_mode,
      email_from: form.email_from.trim(),
      email_smtp_host: form.email_smtp_host.trim(),
      email_smtp_port: form.email_smtp_port,
      email_smtp_username: form.email_smtp_username.trim(),
      email_smtp_password: form.email_smtp_password || undefined,
      email_smtp_tls_mode: form.email_smtp_tls_mode,
    })
    assignSettings(data)
    ElMessage.success(t('common.saved') as string)
  } finally {
    saving.value = false
  }
}

function storageChannelLabel(channel: StorageChannel) {
  const status = channel.status === StorageChannelStatus.Enabled ? '启用' : '停用'
  return `${channel.name || `渠道 #${channel.id}`}（${channel.provider}，${status}）`
}

async function sendTestEmail() {
  if (!testEmail.value.trim()) {
    ElMessage.warning('请填写测试收件邮箱')
    return
  }
  testingEmail.value = true
  try {
    await sendSystemTestEmail(testEmail.value.trim())
    ElMessage.success(form.email_service_mode === 'log' ? '测试验证码已写入服务端日志' : '测试邮件已发送')
  } finally {
    testingEmail.value = false
  }
}

async function generateKey(rotate: boolean) {
  generating.value = true
  try {
    const data = await generateLicenseSigningKey({ rotate })
    assignSettings(data)
    ElMessage.success(t('system_settings.license_key_generated') as string)
  } finally {
    generating.value = false
  }
}

async function confirmRotate() {
  await ElMessageBox.confirm(
    t('system_settings.rotate_license_key_confirm') as string,
    t('common.confirm') as string,
    { type: 'warning' },
  )
  await generateKey(true)
}

async function copyPublicKey() {
  if (!licenseSigning.public_key_b64) return
  await navigator.clipboard.writeText(licenseSigning.public_key_b64)
  ElMessage.success(t('common.copied') as string)
}

async function copyPrivateKey() {
  if (!licenseSigning.private_key_b64) return
  await navigator.clipboard.writeText(licenseSigning.private_key_b64)
  ElMessage.success(t('common.copied') as string)
}

onMounted(load)
</script>
