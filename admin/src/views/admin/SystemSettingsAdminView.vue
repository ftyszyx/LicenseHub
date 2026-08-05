<template>
  <div class="space-y-4">
    <el-card shadow="hover">
      <div class="flex items-center justify-between gap-3">
        <h2 class="text-xl font-semibold">{{ $t('system_settings.title') }}</h2>
        <el-button type="primary" :loading="saving" @click="submit">{{ $t('common.save') }}</el-button>
      </div>
    </el-card>

    <el-card shadow="never">
      <template #header><h3 class="text-base font-semibold">分销设置</h3></template>
      <el-form v-loading="loading" :model="form" label-width="170px" class="max-w-3xl">
        <el-form-item label="开放分销功能">
          <el-switch v-model="form.distribution_enabled" />
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
import { fetchSystemSettings, generateLicenseSigningKey, updateSystemSettings } from '@/apis/system_settings'
import type { LicenseSigningInfo, SaveSystemSettingsReq, SiteSettings } from '@/types'

const { t } = useI18n()
const formRef = ref<FormInstance>()
const loading = ref(false)
const saving = ref(false)
const generating = ref(false)

type SettingsForm = SaveSystemSettingsReq & {
  distribution_default_rate_percent: number
  distribution_min_withdraw_yuan: number
}

const form = reactive<SettingsForm>({
  storefront_title: 'LicenseHub',
  distribution_enabled: false,
  distribution_default_rate_bps: 2000,
  distribution_attribution_days: 30,
  distribution_holding_days: 7,
  distribution_min_withdraw_cents: 5000,
  distribution_default_rate_percent: 20,
  distribution_min_withdraw_yuan: 50,
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
    assignSettings(data)
  } finally {
    loading.value = false
  }
}

function assignSettings(data: SiteSettings) {
  form.storefront_title = data.storefront_title || 'LicenseHub'
  form.distribution_enabled = Boolean(data.distribution?.enabled)
  form.distribution_default_rate_bps = data.distribution?.default_rate_bps ?? 2000
  form.distribution_attribution_days = data.distribution?.attribution_days ?? 30
  form.distribution_holding_days = data.distribution?.holding_days ?? 7
  form.distribution_min_withdraw_cents = data.distribution?.min_withdraw_cents ?? 5000
  form.distribution_default_rate_percent = form.distribution_default_rate_bps / 100
  form.distribution_min_withdraw_yuan = form.distribution_min_withdraw_cents / 100
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
      distribution_enabled: form.distribution_enabled,
      distribution_default_rate_bps: Math.round(form.distribution_default_rate_percent * 100),
      distribution_attribution_days: form.distribution_attribution_days,
      distribution_holding_days: form.distribution_holding_days,
      distribution_min_withdraw_cents: Math.round(form.distribution_min_withdraw_yuan * 100),
    })
    assignSettings(data)
    ElMessage.success(t('common.saved') as string)
  } finally {
    saving.value = false
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
