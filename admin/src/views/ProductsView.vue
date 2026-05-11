<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import ProductCatalog from '@/components/ProductCatalog.vue'

const route = useRoute()

const appId = computed(() => {
  const value = route.params.appid
  const raw = Array.isArray(value) ? value[0] : value
  if (!raw) return null
  const parsed = Number(raw)
  return Number.isFinite(parsed) ? parsed : null
})
</script>

<template>
  <ProductCatalog :app-id="appId" :grouped="!appId" :title="appId ? '' : '全部商品'" />
</template>
