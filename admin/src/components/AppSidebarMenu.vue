<template>
  <div class="flex h-full flex-col bg-white">
    <div class="border-b border-slate-100 px-5 py-5">
      <div class="flex items-center gap-3">
        <span class="flex h-10 w-10 items-center justify-center rounded-xl bg-slate-950 text-sm font-semibold text-white">LH</span>
        <div class="min-w-0">
          <div class="truncate text-base font-semibold text-slate-950">{{ $t("app.title") }}</div>
          <div class="text-xs text-slate-500">{{ $t("menu.console") }}</div>
        </div>
      </div>
    </div>
    <AppMenu :items="menuItems" :default-openeds="defaultOpeneds" mode="vertical" class="flex-1 overflow-y-auto py-3" />
  </div>
</template>

<script setup lang="ts">
import AppMenu from "./AppMenu.vue";
import type { AdminMenuItem } from "@/types/menu";
import { RoutePath } from "@/types/route";
const baseMenuItems: AdminMenuItem[] = [
  //i18n-key: menu.dashboard
  {
    label: "menu.group_overview",
    icon: "DataLine",
    index: "overview",
    children: [{ label: "menu.dashboard", icon: "House", path: RoutePath.AdminDashboard }],
  },
  {
    label: "menu.group_business",
    icon: "Shop",
    index: "business",
    children: [
      //i18n-key: menu.apps
      { label: "menu.apps", icon: "Cpu", path: RoutePath.AdminApps },
      //i18n-key: menu.products
      { label: "menu.products", icon: "Goods", path: RoutePath.AdminProducts },
      //i18n-key: menu.orders
      { label: "menu.orders", icon: "ShoppingCart", path: RoutePath.AdminOrders },
      { label: "佣金与提现", icon: "Money", path: RoutePath.AdminDistribution },
    ],
  },
  {
    label: "menu.group_license",
    icon: "Key",
    index: "license",
    children: [
      //i18n-key: menu.devices
      { label: "menu.devices", icon: "Iphone", path: RoutePath.AdminDevices },
      //i18n-key: menu.reg_codes
      { label: "menu.reg_codes", icon: "Tickets", path: RoutePath.AdminRegCodes },
      //i18n-key: menu.reg_test
      { label: "menu.reg_test", icon: "Connection", path: RoutePath.AdminRegTest },
      //i18n-key: menu.use_records
      { label: "menu.use_records", icon: "Memo", path: RoutePath.AdminUseRecords },
    ],
  },
  {
    label: "menu.group_system",
    icon: "Setting",
    index: "system",
    children: [
      //i18n-key: menu.users
      { label: "menu.users", icon: "User", path: RoutePath.AdminUsers },
      //i18n-key: menu.roles
      { label: "menu.roles", icon: "UserFilled", path: RoutePath.AdminRoles },
      //i18n-key: menu.permissions
      { label: "menu.permissions", icon: "Lock", path: RoutePath.AdminPermissions },
      //i18n-key: menu.system_settings
      { label: "menu.system_settings", icon: "Setting", path: RoutePath.AdminSystemSettings },
      //i18n-key: menu.storage_channels
      { label: "menu.storage_channels", icon: "UploadFilled", path: RoutePath.AdminStorageChannels },
      //i18n-key: menu.payment_settings
      { label: "menu.payment_settings", icon: "CreditCard", path: RoutePath.AdminPaymentSettings },
    ],
  },
];

const menuItems = baseMenuItems;

const defaultOpeneds = ["overview", "business", "license", "system"];
</script>

<style scoped></style>
