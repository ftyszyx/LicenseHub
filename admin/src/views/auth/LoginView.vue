<template>
  <div class="flex items-center justify-center min-h-screen bg-[#2d3748]">
    <div class="w-full max-w-sm p-8 space-y-6 bg-[#4a5568] rounded-lg shadow-lg">
      <div class="flex justify-center">
        <svg
          class="w-16 h-16 text-white"
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="1.5"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M7.864 4.243A7.5 7.5 0 0119.5 10.5c0 2.92-.556 5.709-1.588 8.188a7.5 7.5 0 01-11.828 0A7.5 7.5 0 017.864 4.243zM12 15.75a3.75 3.75 0 100-7.5 3.75 3.75 0 000 7.5z"
          />
        </svg>
      </div>
      <h2 class="text-2xl font-bold text-center text-white">{{ $t('auth.welcome') }}</h2>
      <form @submit.prevent="handleLogin" class="space-y-6">
        <div>
          <input
            type="text"
            :placeholder="$t('auth.username')"
            v-model="username"
            required
            class="w-full px-4 py-2 text-gray-900 bg-white border border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div class="relative">
          <input
            :type="passwordFieldType"
            :placeholder="$t('auth.password')"
            v-model="password"
            required
            class="w-full px-4 py-2 text-gray-900 bg-white border border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <span class="absolute inset-y-0 right-0 flex items-center pr-3 cursor-pointer" @click="togglePasswordVisibility">
             <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5 text-gray-500">
                <path stroke-linecap="round" stroke-linejoin="round" d="M2.036 12.322a1.012 1.012 0 010-.639l4.43-6.112a1.011 1.011 0 011.64 0l4.43 6.11a1.012 1.012 0 010 .64l-4.43 6.11a1.012 1.012 0 01-1.64 0l-4.43-6.111zM15.121 12a3.121 3.121 0 11-6.242 0 3.121 3.121 0 016.242 0z" />
            </svg>
          </span>
        </div>
        <button
          type="submit"
          class="w-full px-4 py-2 text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800 focus:ring-blue-500"
        >
          {{ $t('auth.login') }}
        </button>
        <div v-if="registrationEnabled" class="text-center">
          <router-link to="/register" class="text-sm text-blue-400 hover:underline">{{ $t('auth.no_account_register') }}</router-link>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { RoutePath } from '@/types'
import { useI18n } from 'vue-i18n'
import { fetchSiteSettings } from '@/apis/system_settings'

const username = ref('')
const password = ref('')
const showPassword = ref(false)
const registrationEnabled = ref(false)

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
useI18n()

const passwordFieldType = computed(() => (showPassword.value ? 'text' : 'password'))

const togglePasswordVisibility = () => {
  showPassword.value = !showPassword.value
}

const handleLogin = async () => {
    await authStore.login({username: username.value, password: password.value})
    const redirect = typeof route.query.redirect === 'string' && route.query.redirect.startsWith('/')
      ? route.query.redirect
      : null
    router.push(redirect || (authStore.isAdmin ? RoutePath.AdminDashboard : RoutePath.UserHome))
}

onMounted(async () => {
  try {
    registrationEnabled.value = Boolean((await fetchSiteSettings()).registration_enabled)
  } catch {
    registrationEnabled.value = false
  }
})
</script>
