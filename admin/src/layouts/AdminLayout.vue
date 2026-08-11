<template>
  <div class="flex h-screen bg-slate-100">
    <aside class="flex w-64 flex-col border-r border-slate-200 bg-white">
      <AppSidebarMenu class="flex-1" />
    </aside>
    <main class="admin-main p-6">
      <div class="admin-toolbar mb-4 flex items-center justify-end gap-3">
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
      <div class="admin-content">
        <RouterView />
      </div>
    </main>
    <el-dialog v-model="pwdVisible" :title="$t('auth.change_password')" width="420px">
      <el-form label-width="130px">
        <el-form-item :label="$t('auth.old_password')"><el-input v-model="oldPwd" type="password" show-password /></el-form-item>
        <el-form-item :label="$t('auth.new_password')"><el-input v-model="newPwd" type="password" show-password /></el-form-item>
        <el-form-item :label="$t('auth.confirm_password')"><el-input v-model="confirmPwd" type="password" show-password /></el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="pwdVisible=false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submitChangePwd">{{ $t('common.confirm') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { RouterView, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import AppSidebarMenu from '@/components/AppSidebarMenu.vue'
import { storeToRefs } from 'pinia'
import { useLocaleStore } from '@/stores/locale'
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { changeMyPassword } from '@/apis/auth'
import { ElMessage } from 'element-plus'

const authStore = useAuthStore()
const { t } = useI18n()
const router = useRouter()
const localeStore = useLocaleStore()
const { current: locale } = storeToRefs(localeStore)
const currentLocaleLabel = computed(() => locale.value === 'zh-cn' ? String(t('common.lang_zh_cn')) : String(t('common.lang_en')))

const handleLogout = () => {
  authStore.logout()
  router.push('/login')
}

const pwdVisible = ref(false)
const oldPwd = ref('')
const newPwd = ref('')
const confirmPwd = ref('')
function openChangePwd(){ pwdVisible.value=true }
async function submitChangePwd(){
  if(!newPwd.value || newPwd.value !== confirmPwd.value){ ElMessage.error(String(t('auth.password_mismatch'))); return }
  await changeMyPassword({ old_password: oldPwd.value, new_password: newPwd.value })
  ElMessage.success(String(t('auth.change_password_success')))
  pwdVisible.value=false; oldPwd.value=''; newPwd.value=''; confirmPwd.value=''
}

function onLangCommand(cmd: string){ localeStore.setLocale(cmd as any) }
function onSettingsCommand(cmd: string){ if(cmd==='pwd'){ openChangePwd() } else if(cmd==='logout'){ handleLogout() } }
</script>

<style scoped>
.admin-main {
  display: flex;
  min-width: 0;
  flex: 1 1 0%;
  flex-direction: column;
  overflow: hidden;
}

.admin-toolbar {
  flex: none;
}

.admin-content {
  min-height: 0;
  flex: 1 1 0%;
  overflow: auto;
}
</style>
