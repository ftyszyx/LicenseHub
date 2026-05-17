<template>
  <div class="space-y-4">
    <el-card shadow="hover">
      <div class="flex items-center justify-between gap-3">
        <h2 class="text-xl font-semibold">{{ $t('system_settings.title') }}</h2>
        <el-button type="primary" :loading="saving" @click="submit">{{ $t('common.save') }}</el-button>
      </div>
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
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { fetchSystemSettings, updateSystemSettings } from '@/apis/system_settings'
import type { SaveSystemSettingsReq } from '@/types'

const { t } = useI18n()
const formRef = ref<FormInstance>()
const loading = ref(false)
const saving = ref(false)

const form = reactive<SaveSystemSettingsReq>({
  storefront_title: 'LicenseHub',
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
    form.storefront_title = data.storefront_title || 'LicenseHub'
  } finally {
    loading.value = false
  }
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
    })
    form.storefront_title = data.storefront_title
    ElMessage.success(t('common.saved') as string)
  } finally {
    saving.value = false
  }
}

onMounted(load)
</script>
