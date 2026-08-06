<template>
  <div class="space-y-5">
    <h2 class="text-xl font-semibold text-slate-950">首页</h2>

    <section v-loading="loading" class="rounded-lg border border-slate-200 bg-white p-6">
      <div class="flex items-center gap-4 border-b border-slate-100 pb-5">
        <span class="flex h-12 w-12 items-center justify-center rounded-lg bg-emerald-600 text-white">
          <el-icon :size="24"><UserFilled /></el-icon>
        </span>
        <div class="min-w-0">
          <div class="truncate text-lg font-semibold text-slate-950">{{ user?.username || '-' }}</div>
          <el-tag class="mt-1" type="success" effect="plain">账号正常</el-tag>
        </div>
      </div>

      <el-descriptions class="mt-5" :column="2" border>
        <el-descriptions-item label="用户 ID">{{ user?.id ?? '-' }}</el-descriptions-item>
        <el-descriptions-item label="邮箱">{{ user?.email || '历史账号暂未绑定邮箱' }}</el-descriptions-item>
        <el-descriptions-item label="注册时间">{{ formatDate(user?.created_at) }}</el-descriptions-item>
      </el-descriptions>
    </section>

    <section class="rounded-lg border border-slate-200 bg-white p-6">
      <h3 class="text-base font-semibold text-slate-950">快捷入口</h3>
      <div class="mt-4 flex flex-wrap gap-3">
        <el-button type="primary" :icon="ShoppingBag" @click="router.push(RoutePath.Home)">返回商城</el-button>
        <el-button :icon="Tickets" @click="router.push(RoutePath.UserOrders)">我的订单</el-button>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ShoppingBag, Tickets } from '@element-plus/icons-vue'
import { sentGetUserInfo } from '@/apis/auth'
import { RoutePath } from '@/types'
import type { UserModel } from '@/types'

const router = useRouter()
const loading = ref(false)
const user = ref<UserModel>()

function formatDate(value?: string) {
  if (!value) return '-'
  return new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(value))
}

async function load() {
  loading.value = true
  try {
    user.value = await sentGetUserInfo()
  } finally {
    loading.value = false
  }
}

onMounted(load)
</script>
