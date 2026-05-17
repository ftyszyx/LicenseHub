<template>
  <template v-for="(item, idx) in items" :key="item.index ?? item.path ?? String(idx)">
    <el-sub-menu v-if="item.children && item.children.length" :index="item.index ?? String(idx)">
      <template #title>
        <el-icon v-if="item.icon" class="mr-2">
          <component :is="item.icon" />
        </el-icon>
        <span>{{ $t(item.label) }}</span>
      </template>
      <AppMenuSub :items="item.children" />
    </el-sub-menu>
    <el-menu-item v-else :index="item.index ?? (item.path ?? String(idx))" :route="item.path" :disabled="item.disabled">
      <el-icon v-if="item.icon" class="mr-2">
        <component :is="item.icon" />
      </el-icon>
      <span>{{ $t(item.label) }}</span>
    </el-menu-item>
  </template>
</template>

<script setup lang="ts">
import type { AdminMenuItem } from '@/types/menu'
defineProps<{ items: AdminMenuItem[] }>()
</script>

<style scoped>
</style>


