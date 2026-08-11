<template>
  <div class="space-y-4">
    <el-card shadow="hover">
      <div class="flex items-center justify-between gap-4">
        <div>
          <h2 class="text-xl font-semibold">{{ $t('reg_test.title') }}</h2>
          <p class="text-sm text-gray-500 mt-1">{{ $t('reg_test.subtitle') }}</p>
        </div>
      </div>
    </el-card>

    <el-row :gutter="16">
      <el-col :xs="24" :xl="8">
        <el-card shadow="never" class="h-full">
          <template #header>
            <span class="font-medium">{{ $t('reg_test.bind_title') }}</span>
          </template>

          <el-form label-width="100px">
            <el-form-item :label="$t('reg_codes.app')">
              <el-select v-model.number="bindForm.app_id" filterable clearable class="w-full">
                <el-option v-for="opt in appOptions" :key="opt.id" :label="opt.name" :value="opt.id" />
              </el-select>
            </el-form-item>

            <el-form-item :label="$t('apps.valid_key')">
              <el-select v-model.number="bindForm.app_id" filterable clearable class="w-full">
                <el-option v-for="opt in appOptions" :key="opt.id" :label="opt.app_valid_key" :value="opt.id" />
              </el-select>
            </el-form-item>

            <el-form-item :label="$t('reg_codes.code')">
              <el-input v-model="bindForm.reg_code" clearable />
            </el-form-item>

            <el-form-item :label="$t('reg_codes.device_id')">
              <el-select v-model="bindForm.device_id" filterable allow-create default-first-option clearable class="w-full">
                <el-option v-for="opt in getDeviceOptions(bindForm.app_id)" :key="opt.id" :label="opt.device_id" :value="opt.device_id" />
              </el-select>
            </el-form-item>

            <el-form-item>
              <el-button type="primary" :loading="bindLoading" @click="submitBind">{{ $t('reg_test.bind_submit') }}</el-button>
              <el-button @click="resetBind">{{ $t('common.reset') }}</el-button>
            </el-form-item>
          </el-form>

          <el-empty v-if="!bindResult" :description="$t('reg_test.empty_result')" :image-size="60" />

          <el-descriptions v-else :column="1" border size="small">
            <el-descriptions-item v-if="bindResult.expire_time" :label="$t('devices.expire_time')">
              {{ formatTimestamp(bindResult.expire_time) }}
            </el-descriptions-item>
            <el-descriptions-item v-if="bindResult.remain_count != null" :label="$t('devices.remaining')">
              {{ bindResult.remain_count }}
            </el-descriptions-item>
          </el-descriptions>
        </el-card>
      </el-col>

      <el-col :xs="24" :xl="8">
        <el-card shadow="never" class="h-full">
          <template #header>
            <span class="font-medium">{{ $t('reg_test.check_title') }}</span>
          </template>

          <el-form label-width="100px">
            <el-form-item :label="$t('reg_codes.app')">
              <el-select v-model.number="checkForm.app_id" filterable clearable class="w-full">
                <el-option v-for="opt in appOptions" :key="opt.id" :label="opt.name" :value="opt.id" />
              </el-select>
            </el-form-item>

            <el-form-item :label="$t('apps.valid_key')">
              <el-select v-model.number="checkForm.app_id" filterable clearable class="w-full">
                <el-option v-for="opt in appOptions" :key="opt.id" :label="opt.app_valid_key" :value="opt.id" />
              </el-select>
            </el-form-item>

            <el-form-item :label="$t('reg_codes.device_id')">
              <el-select v-model="checkForm.device_id" filterable allow-create default-first-option clearable class="w-full">
                <el-option v-for="opt in getDeviceOptions(checkForm.app_id)" :key="opt.id" :label="opt.device_id" :value="opt.device_id" />
              </el-select>
            </el-form-item>

            <el-form-item>
              <el-button type="success" :loading="checkLoading" @click="submitCheck">{{ $t('reg_test.check_submit') }}</el-button>
              <el-button @click="resetCheck">{{ $t('common.reset') }}</el-button>
            </el-form-item>
          </el-form>

          <el-empty v-if="!checkResult" :description="$t('reg_test.empty_result')" :image-size="60" />

          <el-descriptions v-else :column="1" border size="small">
            <el-descriptions-item v-if="checkResult.expire_time" :label="$t('devices.expire_time')">
              {{ formatTimestamp(checkResult.expire_time) }}
            </el-descriptions-item>
            <el-descriptions-item v-if="checkResult.remain_count != null" :label="$t('devices.remaining')">
              {{ checkResult.remain_count }}
            </el-descriptions-item>
          </el-descriptions>
        </el-card>
      </el-col>

      <el-col :xs="24" :xl="8">
        <el-card shadow="never" class="h-full">
          <template #header>
            <span class="font-medium">{{ $t('reg_test.usecount_title') }}</span>
          </template>

          <el-form label-width="100px">
            <el-form-item :label="$t('reg_codes.app')">
              <el-select v-model.number="useForm.app_id" filterable clearable class="w-full">
                <el-option v-for="opt in appOptions" :key="opt.id" :label="opt.name" :value="opt.id" />
              </el-select>
            </el-form-item>

            <el-form-item :label="$t('apps.valid_key')">
              <el-select v-model.number="useForm.app_id" filterable clearable class="w-full">
                <el-option v-for="opt in appOptions" :key="opt.id" :label="opt.app_valid_key" :value="opt.id" />
              </el-select>
            </el-form-item>

            <el-form-item :label="$t('reg_codes.device_id')">
              <el-select v-model="useForm.device_id" filterable allow-create default-first-option clearable class="w-full">
                <el-option v-for="opt in getDeviceOptions(useForm.app_id)" :key="opt.id" :label="opt.device_id" :value="opt.device_id" />
              </el-select>
            </el-form-item>

            <el-form-item :label="$t('reg_test.use_count')">
              <el-input-number v-model="useForm.use_count" :min="1" class="w-full" />
            </el-form-item>

            <el-form-item :label="$t('reg_test.use_info')">
              <el-input v-model="useForm.use_info_text" type="textarea" :rows="4" />
            </el-form-item>

            <el-form-item>
              <el-button type="warning" :loading="useLoading" @click="submitUseCount">{{ $t('reg_test.usecount_submit') }}</el-button>
              <el-button @click="resetUseCount">{{ $t('common.reset') }}</el-button>
            </el-form-item>
          </el-form>

          <el-empty v-if="useResult == null" :description="$t('reg_test.empty_result')" :image-size="60" />

          <el-descriptions v-else :column="1" border size="small">
            <el-descriptions-item :label="$t('reg_test.remain_count')">
              {{ useResult.remain_count }}
            </el-descriptions-item>
          </el-descriptions>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { useI18n } from 'vue-i18n'

import { fetchApps } from '@/apis/apps'
import { fetchDevices } from '@/apis/devices'
import { bindRegCode, checkRegDevice, useRegCount } from '@/apis/reg_codes'
import type { AppModel } from '@/types/apps'
import type { DeviceInfo } from '@/types/app_devices'
import type { RegCodeBindCheckResp, UseCountResp } from '@/types/reg_codes'
import { formatTime } from '@/utils'
import { fetchAllPages } from '@/utils/pagination'

const { t } = useI18n()

const appOptions = ref<AppModel[]>([])
const deviceOptionsMap = ref<Record<number, DeviceInfo[]>>({})

const bindLoading = ref(false)
const checkLoading = ref(false)
const useLoading = ref(false)
const bindResult = ref<RegCodeBindCheckResp | null>(null)
const checkResult = ref<RegCodeBindCheckResp | null>(null)
const useResult = ref<UseCountResp | null>(null)

const bindForm = reactive({
  app_id: 0,
  reg_code: '',
  device_id: '',
})

const checkForm = reactive({
  app_id: 0,
  device_id: '',
})

const useForm = reactive({
  app_id: 0,
  device_id: '',
  use_count: 1,
  use_info_text: '',
})

const bindAppKey = computed(() => appOptions.value.find(item => item.id === bindForm.app_id)?.app_valid_key ?? '')
const checkAppKey = computed(() => appOptions.value.find(item => item.id === checkForm.app_id)?.app_valid_key ?? '')
const useAppKey = computed(() => appOptions.value.find(item => item.id === useForm.app_id)?.app_valid_key ?? '')

function getDeviceOptions(appId: number) {
  return appId ? (deviceOptionsMap.value[appId] ?? []) : []
}

function formatTimestamp(value: number | null | undefined) {
  return value ? formatTime(new Date(value * 1000).toISOString()) : '-'
}

async function ensureDevicesLoaded(appId: number) {
  if (!appId || deviceOptionsMap.value[appId]) {
    return
  }
  const devices = await fetchAllPages(params => fetchDevices({ ...params, app_id: appId }))
  deviceOptionsMap.value = {
    ...deviceOptionsMap.value,
    [appId]: devices,
  }
}

function syncDeviceValue(appId: number, currentDeviceId: string) {
  if (!currentDeviceId) {
    return currentDeviceId
  }
  const exists = getDeviceOptions(appId).some(item => item.device_id === currentDeviceId)
  return exists ? currentDeviceId : ''
}

function resetBind() {
  bindForm.reg_code = ''
  bindForm.device_id = ''
  bindResult.value = null
}

function resetCheck() {
  checkForm.device_id = ''
  checkResult.value = null
}

function resetUseCount() {
  useForm.device_id = ''
  useForm.use_count = 1
  useForm.use_info_text = ''
  useResult.value = null
}

async function submitBind() {
  if (!bindAppKey.value || !bindForm.reg_code || !bindForm.device_id) {
    ElMessage.warning(t('common.please_check_form') as string)
    return
  }

  bindLoading.value = true
  try {
    bindResult.value = await bindRegCode({
      app_key: bindAppKey.value,
      reg_code: bindForm.reg_code,
      device_id: bindForm.device_id,
    })
    checkForm.app_id = bindForm.app_id
    checkForm.device_id = bindForm.device_id
    useForm.app_id = bindForm.app_id
    useForm.device_id = bindForm.device_id
    ElMessage.success(t('reg_test.bind_success') as string)
  } finally {
    bindLoading.value = false
  }
}

async function submitCheck() {
  if (!checkAppKey.value || !checkForm.device_id) {
    ElMessage.warning(t('common.please_check_form') as string)
    return
  }

  checkLoading.value = true
  try {
    checkResult.value = await checkRegDevice({
      app_key: checkAppKey.value,
      device_id: checkForm.device_id,
    })
    ElMessage.success(t('reg_test.check_success') as string)
  } finally {
    checkLoading.value = false
  }
}

async function submitUseCount() {
  if (!useAppKey.value || !useForm.device_id || !useForm.use_count) {
    ElMessage.warning(t('common.please_check_form') as string)
    return
  }

  let useInfo: any = null
  if (useForm.use_info_text.trim()) {
    try {
      useInfo = JSON.parse(useForm.use_info_text)
    } catch {
      ElMessage.error(t('reg_test.invalid_use_info') as string)
      return
    }
  }

  useLoading.value = true
  try {
    useResult.value = await useRegCount({
      app_key: useAppKey.value,
      device_id: useForm.device_id,
      use_count: useForm.use_count,
      use_info: useInfo,
    })
    checkForm.app_id = useForm.app_id
    checkForm.device_id = useForm.device_id
    ElMessage.success(t('reg_test.usecount_success') as string)
  } finally {
    useLoading.value = false
  }
}

async function loadApps() {
  appOptions.value = await fetchAllPages(fetchApps)

  if ((!bindForm.app_id || bindForm.app_id === 0) && appOptions.value.length) {
    bindForm.app_id = appOptions.value[0].id
  }
  if ((!checkForm.app_id || checkForm.app_id === 0) && appOptions.value.length) {
    checkForm.app_id = appOptions.value[0].id
  }
  if ((!useForm.app_id || useForm.app_id === 0) && appOptions.value.length) {
    useForm.app_id = appOptions.value[0].id
  }
}

onMounted(async () => {
  await loadApps()
})

watch(
  () => bindForm.app_id,
  async (appId) => {
    await ensureDevicesLoaded(appId)
    bindForm.device_id = syncDeviceValue(appId, bindForm.device_id)
  },
  { immediate: true }
)

watch(
  () => checkForm.app_id,
  async (appId) => {
    await ensureDevicesLoaded(appId)
    checkForm.device_id = syncDeviceValue(appId, checkForm.device_id)
  },
  { immediate: true }
)

watch(
  () => useForm.app_id,
  async (appId) => {
    await ensureDevicesLoaded(appId)
    useForm.device_id = syncDeviceValue(appId, useForm.device_id)
  },
  { immediate: true }
)
</script>

<style scoped></style>
