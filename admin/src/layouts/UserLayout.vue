<template>
  <div class="flex h-screen bg-slate-100">
    <aside class="flex w-60 flex-col border-r border-slate-200 bg-white">
      <UserSidebarMenu class="flex-1" />
    </aside>
    <main class="flex-1 overflow-auto p-6">
      <div class="mb-4 flex items-center justify-end gap-3">
        <el-dropdown @command="onLangCommand">
          <el-button text>{{ currentLocaleLabel }}</el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="en">{{ $t('common.lang_en') }}</el-dropdown-item>
              <el-dropdown-item command="zh-cn">{{ $t('common.lang_zh_cn') }}</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        <el-dropdown @command="onSettingsCommand">
          <el-button text>{{ $t('common.settings') }}</el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="pwd">{{ $t('auth.change_password') }}</el-dropdown-item>
              <el-dropdown-item divided command="logout">{{ $t('auth.logout') }}</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
      <RouterView />
    </main>

    <el-dialog v-model="pwdVisible" :title="$t('auth.change_password')" width="420px">
      <el-form label-width="130px">
        <el-form-item :label="$t('auth.old_password')"><el-input v-model="oldPwd" type="password" show-password /></el-form-item>
        <el-form-item :label="$t('auth.new_password')"><el-input v-model="newPwd" type="password" show-password /></el-form-item>
        <el-form-item :label="$t('auth.confirm_password')"><el-input v-model="confirmPwd" type="password" show-password /></el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="pwdVisible = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submitChangePwd">{{ $t('common.confirm') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterView, useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { ElMessage } from 'element-plus'
import { useI18n } from 'vue-i18n'
import UserSidebarMenu from '@/components/UserSidebarMenu.vue'
import { useAuthStore } from '@/stores/auth'
import { useLocaleStore } from '@/stores/locale'
import { changeMyPassword } from '@/apis/auth'
import { RoutePath } from '@/types'

const authStore = useAuthStore()
const router = useRouter()
const localeStore = useLocaleStore()
const { current: locale } = storeToRefs(localeStore)
const { t } = useI18n()
const currentLocaleLabel = computed(() => locale.value === 'zh-cn' ? String(t('common.lang_zh_cn')) : String(t('common.lang_en')))

const pwdVisible = ref(false)
const oldPwd = ref('')
const newPwd = ref('')
const confirmPwd = ref('')

async function submitChangePwd() {
  if (!newPwd.value || newPwd.value !== confirmPwd.value) {
    ElMessage.error(String(t('auth.password_mismatch')))
    return
  }
  await changeMyPassword({ old_password: oldPwd.value, new_password: newPwd.value })
  ElMessage.success(String(t('auth.change_password_success')))
  pwdVisible.value = false
  oldPwd.value = ''
  newPwd.value = ''
  confirmPwd.value = ''
}

function onLangCommand(command: string) { localeStore.setLocale(command as any) }
function onSettingsCommand(command: string) {
  if (command === 'pwd') pwdVisible.value = true
  if (command === 'logout') {
    authStore.logout()
    router.push(RoutePath.Login)
  }
}
</script>
