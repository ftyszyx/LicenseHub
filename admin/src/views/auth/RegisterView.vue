<template>
  <div class="flex items-center justify-center min-h-screen bg-[#2d3748]">
    <div class="w-full max-w-sm p-8 space-y-6 bg-[#4a5568] rounded-lg shadow-lg">
      <div class="flex justify-center">
        <svg class="w-16 h-16 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="M19 7.5v3m0 0v3m0-3h3m-3 0h-3m-2.25-4.125a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zM4 19.235v-.11a6.375 6.375 0 0112.75 0v.109A12.318 12.318 0 0110.374 21c-2.331 0-4.512-.645-6.374-1.766z" />
        </svg>
      </div>
      <h2 class="text-2xl font-bold text-center text-white">{{ $t('auth.register') }}</h2>
      <form @submit.prevent="handleRegister" class="space-y-6">
        <div>
          <input type="text" :placeholder="$t('auth.username')" v-model="username" required class="w-full px-4 py-2 text-gray-900 bg-white border border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500" />
        </div>
        <div class="relative">
          <input :type="passwordFieldType" :placeholder="$t('auth.password')" v-model="password" required class="w-full px-4 py-2 text-gray-900 bg-white border border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500" />
        </div>
        <div class="relative">
          <input :type="passwordFieldType" :placeholder="$t('auth.confirm_password')" v-model="confirmPassword" required class="w-full px-4 py-2 text-gray-900 bg-white border border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500" />
        </div>
        <button type="submit" class="w-full px-4 py-2 text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800 focus:ring-blue-500">
          {{ $t('auth.register') }}
        </button>
        <div class="text-center">
          <router-link to="/login" class="text-sm text-blue-400 hover:underline">{{ $t('auth.have_account_login') }}</router-link>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { ElMessage } from 'element-plus'
import { RoutePath } from '@/types'

const username = ref('')
const password = ref('')
const confirmPassword = ref('')
const showPassword = ref(false)

const router = useRouter()
const authStore = useAuthStore()
const { t } = useI18n()

const passwordFieldType = computed(() => (showPassword.value ? 'text' : 'password'))

const handleRegister = async () => {
  if (password.value !== confirmPassword.value) {
    ElMessage.error(t('auth.password_mismatch'))
    return
  }
  try {
    await authStore.register({ username: username.value, password: password.value })
    router.push(RoutePath.UserHome)
  } catch (error) {
    ElMessage.error(t('auth.register_failed'))
    console.error(error)
  }
}
</script>
