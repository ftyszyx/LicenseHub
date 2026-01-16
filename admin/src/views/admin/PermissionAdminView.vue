<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { ElMessage } from "element-plus";
import { useI18n } from "vue-i18n";
import { fetchPermissions, fetchRolePermissionIds, setRolePermissions } from "@/apis/permissions";
import type { PermissionInfo } from "@/types/permissions";
import type { RoleInfo } from "@/types";
import { fetchRoles } from "@/apis/roles";
const { t } = useI18n();

const roles = ref<RoleInfo[]>([]);
const permissions = ref<PermissionInfo[]>([]);

const selectedRoleId = ref<number | undefined>(undefined);
const selectedPermissionIds = ref<number[]>([]);

const groupedPermissions = computed(() => {
  const map = new Map<string, PermissionInfo[]>();
  for (const p of permissions.value) {
    const key = p.resource || "";
    const arr = map.get(key) ?? [];
    arr.push(p);
    map.set(key, arr);
  }
  for (const arr of map.values()) {
    arr.sort((a, b) => a.action.localeCompare(b.action));
  }
  return Array.from(map.entries()).sort((a, b) => a[0].localeCompare(b[0]));
});

async function loadRolesAndPermissions() {
  const [rolesResp, perms] = await Promise.all([
    fetchRoles({ page: 1, page_size: 1000 }),
    fetchPermissions(),
  ]);
  roles.value = rolesResp.list;
  permissions.value = perms;
  if (!selectedRoleId.value && roles.value.length) {
    selectedRoleId.value = roles.value[0].id;
    await loadRolePermissionIds();
  }
}

async function loadRolePermissionIds() {
  if (!selectedRoleId.value) return;
  const resp = await fetchRolePermissionIds(selectedRoleId.value);
  selectedPermissionIds.value = resp.permission_ids;
}

async function save() {
  if (!selectedRoleId.value) {
    ElMessage.error(String(t("common.please_check_form")));
    return;
  }
  await setRolePermissions(selectedRoleId.value, { permission_ids: selectedPermissionIds.value });
  ElMessage.success(String(t("common.save")));
  await loadRolePermissionIds();
}

onMounted(loadRolesAndPermissions);
</script>

<template>
  <div class="space-y-6">
    <el-card shadow="hover">
      <div class="flex items-center gap-3">
        <div class="w-64">
          <el-select v-model="selectedRoleId" filterable class="w-full" @change="loadRolePermissionIds">
            <el-option v-for="r in roles" :key="r.id" :label="r.name" :value="r.id" />
          </el-select>
        </div>
        <el-button type="primary" @click="save">{{ $t('common.save') }}</el-button>
      </div>
    </el-card>

    <el-card shadow="never">
      <div v-for="([resource, list]) in groupedPermissions" :key="resource" class="mb-6">
        <div class="text-sm font-semibold text-gray-700 mb-2">{{ resource }}</div>
        <el-checkbox-group v-model="selectedPermissionIds">
          <el-checkbox v-for="p in list" :key="p.id" :label="p.id">
            <span class="mr-2">{{ p.action }}</span>
            <span class="text-gray-500">{{ p.name }}</span>
          </el-checkbox>
        </el-checkbox-group>
      </div>
    </el-card>
  </div>
</template>

<style scoped></style>
