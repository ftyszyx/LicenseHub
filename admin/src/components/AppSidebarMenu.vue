<template>
  <div class="flex flex-col h-full">
    <div class="text-2xl font-bold p-5 text-center">{{ $t('app.title') }}</div>
    <AppMenu :items="menuItems" mode="vertical" class="flex-1" />
  </div>
</template>

<script setup lang="ts">
import AppMenu from './AppMenu.vue'
import type { AdminMenuItem } from '@/types/menu'
import { RoutePath } from '@/types/route'



const menuItems: AdminMenuItem[] = [
  //i18n-key: menu.dashboard
  { label: 'menu.dashboard', icon: 'House', path: RoutePath.AdminDashboard },
  //i18n-key: menu.users
  { label: 'menu.users', icon: 'User', path: RoutePath.AdminUsers },
  //i18n-key: menu.roles
  { label: 'menu.roles', icon: 'UserFilled', path: RoutePath.AdminRoles },
  //i18n-key: menu.permissions
  { label: 'menu.permissions', icon: 'Lock', path: RoutePath.AdminPermissions },
  //i18n-key: menu.apps
  { label: 'menu.apps', icon: 'Cpu', path: RoutePath.AdminApps },
  //i18n-key: menu.products
  { label: 'menu.products', icon: 'Goods', path: RoutePath.AdminProducts },
  //i18n-key: menu.orders
  { label: 'menu.orders', icon: 'ShoppingCart', path: RoutePath.AdminOrders },
  //i18n-key: menu.payment_settings
  { label: 'menu.payment_settings', icon: 'CreditCard', path: RoutePath.AdminPaymentSettings },
  //i18n-key: menu.system_settings
  { label: 'menu.system_settings', icon: 'Setting', path: RoutePath.AdminSystemSettings },
  //i18n-key: menu.devices
  { label: 'menu.devices', icon: 'Iphone', path: RoutePath.AdminDevices },
  //i18n-key: menu.reg_codes
  { label: 'menu.reg_codes', icon: 'Tickets', path: RoutePath.AdminRegCodes },
  //i18n-key: menu.reg_test
  { label: 'menu.reg_test', icon: 'Connection', path: RoutePath.AdminRegTest },
  //i18n-key: menu.use_records
  { label: 'menu.use_records', icon: 'Memo', path: RoutePath.AdminUseRecords },
]
</script>

<style scoped>
</style>


