<template>
  <el-menu
    :default-active="activeIndex"
    :default-openeds="defaultOpeneds"
    class="app-menu h-full border-0"
    :mode="mode"
    :router="true"
  >
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
  </el-menu>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import type { AdminMenuItem, AdminMenuMode } from '@/types/menu'
defineProps<{ items: AdminMenuItem[]; mode?: AdminMenuMode; defaultOpeneds?: string[] }>()
const route = useRoute()
const activeIndex = computed(() => route.path)

</script>

<style scoped>
:deep(.el-menu) {
  background: transparent;
}

:deep(.el-sub-menu__title),
:deep(.el-menu-item) {
  height: 42px;
  margin: 2px 10px;
  border-radius: 8px;
  color: #475569;
  font-size: 14px;
}

:deep(.el-sub-menu__title:hover),
:deep(.el-menu-item:hover) {
  background: #f1f5f9;
  color: #0f172a;
}

:deep(.el-menu-item.is-active) {
  background: #eff6ff;
  color: #2563eb;
  font-weight: 600;
}

:deep(.el-sub-menu.is-active > .el-sub-menu__title) {
  color: #0f172a;
  font-weight: 600;
}

:deep(.el-menu--inline .el-menu-item) {
  height: 38px;
  padding-left: 44px !important;
  font-size: 13px;
}

:deep(.el-icon) {
  margin-right: 10px;
}
</style>


