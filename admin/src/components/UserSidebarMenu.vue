<template>
  <div class="flex h-full flex-col bg-white">
    <div class="border-b border-slate-100 px-5 py-5">
      <div class="flex items-center gap-3">
        <span class="flex h-10 w-10 items-center justify-center rounded-lg bg-emerald-600 text-sm font-semibold text-white">LH</span>
        <div class="min-w-0">
          <div class="truncate text-base font-semibold text-slate-950">LicenseHub</div>
          <div class="text-xs text-slate-500">用户中心</div>
        </div>
      </div>
    </div>

    <AppMenu :items="menuItems" :default-openeds="['user']" mode="vertical" class="flex-1 overflow-y-auto py-3" />

    <div class="border-t border-slate-100 p-4">
      <el-button class="w-full" @click="router.push(RoutePath.Home)">返回商城</el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import AppMenu from './AppMenu.vue'
import type { AdminMenuItem } from '@/types/menu'
import { RoutePath } from '@/types/route'
import { fetchSiteSettings } from '@/apis/system_settings'

const router = useRouter()
const distributionEnabled = ref(false)

const menuItems = computed<AdminMenuItem[]>(() => [{
  label: '用户中心',
  icon: 'User',
  index: 'user',
  children: [
    { label: '首页', icon: 'House', path: RoutePath.UserHome },
    ...(distributionEnabled.value
      ? [{ label: '推广中心', icon: 'Promotion', path: RoutePath.UserDistribution }]
      : []),
  ],
}])

onMounted(async () => {
  try {
    distributionEnabled.value = Boolean((await fetchSiteSettings()).distribution?.enabled)
  } catch {
    distributionEnabled.value = false
  }
})
</script>
