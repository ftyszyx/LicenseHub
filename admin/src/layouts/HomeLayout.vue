<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterLink, RouterView } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { fetchSiteSettings } from '@/apis/system_settings'
import { RoutePath } from '@/types/route'

const authStore = useAuthStore()
const storefrontTitle = ref('LicenseHub')

const adminRoute = computed(() => (
  authStore.isAuthenticated
    ? (authStore.isAdmin ? RoutePath.AdminDashboard : RoutePath.UserHome)
    : RoutePath.Login
))

async function loadSiteSettings() {
  try {
    const settings = await fetchSiteSettings()
    storefrontTitle.value = settings.storefront_title || 'LicenseHub'
  } catch {
    storefrontTitle.value = 'LicenseHub'
  }
}

onMounted(loadSiteSettings)
</script>

<template>
  <header class="sticky top-0 z-40 border-b border-slate-200/80 bg-white/95 backdrop-blur">
    <div class="mx-auto max-w-6xl px-4 sm:px-6">
      <nav class="flex h-16 items-center justify-between">
        <RouterLink to="/" class="flex items-center gap-3 text-slate-950">
          <span class="flex h-9 w-9 items-center justify-center rounded-lg bg-slate-950 text-sm font-semibold text-white">
            LH
          </span>
          <span class="text-base font-semibold tracking-normal">{{ storefrontTitle }}</span>
        </RouterLink>

        <div class="flex items-center rounded-lg bg-slate-100 p-1">
          <RouterLink
            to="/"
            class="rounded-md px-3 py-2 text-sm font-medium text-slate-600 transition hover:text-slate-950"
            exact-active-class="bg-white text-slate-950 shadow-sm"
          >
            {{ $t('home.nav_home') }}
          </RouterLink>
          <RouterLink
            :to="RoutePath.OrderQuery"
            class="rounded-md px-3 py-2 text-sm font-medium text-slate-600 transition hover:text-slate-950"
            active-class="bg-white text-slate-950 shadow-sm"
          >
            {{ $t('home.nav_order_query') }}
          </RouterLink>
        </div>

        <RouterLink
          :to="adminRoute"
          class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium text-slate-800 transition hover:border-slate-950 hover:bg-slate-950 hover:text-white"
        >
          {{ $t('home.nav_admin') }}
        </RouterLink>
      </nav>
    </div>
  </header>

  <main class="bg-slate-50">
    <RouterView />
  </main>
</template>

<style>
</style>
