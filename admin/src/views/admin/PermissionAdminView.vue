<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { Check, Key } from '@element-plus/icons-vue'
import { ElMessage, type CheckboxValueType } from 'element-plus'
import { fetchPermissions, fetchRolePermissionIds, setRolePermissions } from '@/apis/permissions'
import { fetchRoles } from '@/apis/roles'
import type { PermissionInfo, RoleInfo } from '@/types'
import { fetchAllPages } from '@/utils/pagination'

type PermissionGroup = {
  resource: string
  permissions: PermissionInfo[]
}

const RESOURCE_LABELS: Record<string, string> = {
  apps: '应用管理',
  dashboard: '仪表盘',
  devices: '设备管理',
  distribution: '佣金管理',
  me: '个人账户',
  orders: '订单管理',
  payment_settings: '支付设置',
  reg_codes: '注册码管理',
  roles: '角色管理',
  storage_channels: '同步渠道',
  system_settings: '系统设置',
  use_records: '使用记录',
  users: '用户管理',
  version_sync: '版本同步',
}

const ROLE_LABELS: Record<string, string> = {
  admin: '管理员',
  user: '普通用户',
  guest: '访客',
}

const ACTION_LABELS: Record<string, string> = {
  READ: '查看',
  CREATE: '新增',
  UPDATE: '编辑',
  DELETE: '删除',
}

const RESOURCE_ORDER = [
  'dashboard',
  'apps',
  'distribution',
  'orders',
  'users',
  'roles',
  'reg_codes',
  'devices',
  'use_records',
  'payment_settings',
  'storage_channels',
  'version_sync',
  'system_settings',
  'me',
]
const ACTION_ORDER = ['READ', 'CREATE', 'UPDATE', 'DELETE']

const loading = ref(false)
const saving = ref(false)
const roles = ref<RoleInfo[]>([])
const permissions = ref<PermissionInfo[]>([])
const selectedRoleId = ref<number>()
const selectedPermissionIds = ref<number[]>([])

const wildcardPermission = computed(() => permissions.value.find((permission) => permission.resource === '*' || permission.action === '*'))
const granularPermissions = computed(() => permissions.value.filter((permission) => permission.resource !== '*' && permission.action !== '*'))
const selectedRole = computed(() => roles.value.find((role) => role.id === selectedRoleId.value))
const selectedRoleIsAdministrator = computed(() => selectedRole.value?.name === 'admin')
const wildcardSelected = computed(() => (
  Boolean(wildcardPermission.value && selectedPermissionIds.value.includes(wildcardPermission.value.id))
))
const permissionsLocked = computed(() => selectedRoleIsAdministrator.value || wildcardSelected.value)
const selectedPermissionCount = computed(() => {
  if (permissionsLocked.value) return permissions.value.length
  const knownIds = new Set(permissions.value.map((permission) => permission.id))
  return selectedPermissionIds.value.filter((id) => knownIds.has(id)).length
})
const permissionSummary = computed(() => {
  if (selectedRoleIsAdministrator.value) return `管理员固定拥有全部 ${permissions.value.length} 项权限`
  if (wildcardSelected.value) return `已启用全部 ${permissions.value.length} 项权限`
  return `已选择 ${selectedPermissionCount.value} / ${permissions.value.length} 项`
})

const actionColumns = computed(() => {
  const actions = new Set(granularPermissions.value.map((permission) => permission.action))
  return Array.from(actions).sort((a, b) => {
    const aIndex = ACTION_ORDER.indexOf(a)
    const bIndex = ACTION_ORDER.indexOf(b)
    if (aIndex === -1 && bIndex === -1) return a.localeCompare(b)
    if (aIndex === -1) return 1
    if (bIndex === -1) return -1
    return aIndex - bIndex
  })
})

const permissionGroups = computed<PermissionGroup[]>(() => {
  const groups = new Map<string, PermissionInfo[]>()
  for (const permission of granularPermissions.value) {
    const list = groups.get(permission.resource) ?? []
    list.push(permission)
    groups.set(permission.resource, list)
  }

  return Array.from(groups.entries())
    .map(([resource, list]) => ({ resource, permissions: list }))
    .sort((a, b) => {
      const aIndex = RESOURCE_ORDER.indexOf(a.resource)
      const bIndex = RESOURCE_ORDER.indexOf(b.resource)
      if (aIndex === -1 && bIndex === -1) return a.resource.localeCompare(b.resource)
      if (aIndex === -1) return 1
      if (bIndex === -1) return -1
      return aIndex - bIndex
    })
})

const allGranularSelected = computed(() => (
  permissionsLocked.value
  || (granularPermissions.value.length > 0
  && granularPermissions.value.every((permission) => selectedPermissionIds.value.includes(permission.id))
  )
))
const selectedGranularCount = computed(() => (
  permissionsLocked.value
    ? granularPermissions.value.length
    : granularPermissions.value.filter((permission) => selectedPermissionIds.value.includes(permission.id)).length
))
const allGranularIndeterminate = computed(() => (
  !permissionsLocked.value
  && selectedGranularCount.value > 0
  && selectedGranularCount.value < granularPermissions.value.length
))

function resourceLabel(resource: string) {
  return RESOURCE_LABELS[resource] ?? resource
}

function actionLabel(action: string) {
  return ACTION_LABELS[action] ?? action
}

function roleLabel(role: RoleInfo) {
  const label = ROLE_LABELS[role.name] ?? role.description ?? role.name
  return label === role.name ? role.name : `${label}（${role.name}）`
}

function permissionFor(group: PermissionGroup, action: string) {
  return group.permissions.find((permission) => permission.action === action)
}

function isPermissionSelected(permissionId: number) {
  return permissionsLocked.value || selectedPermissionIds.value.includes(permissionId)
}

function togglePermission(permissionId: number, checked: CheckboxValueType) {
  const ids = new Set(selectedPermissionIds.value)
  if (checked) ids.add(permissionId)
  else ids.delete(permissionId)
  selectedPermissionIds.value = Array.from(ids)
}

function selectedGroupCount(group: PermissionGroup) {
  return group.permissions.filter((permission) => isPermissionSelected(permission.id)).length
}

function isGroupSelected(group: PermissionGroup) {
  return permissionsLocked.value || (group.permissions.length > 0 && selectedGroupCount(group) === group.permissions.length)
}

function isGroupIndeterminate(group: PermissionGroup) {
  if (permissionsLocked.value) return false
  const count = selectedGroupCount(group)
  return count > 0 && count < group.permissions.length
}

function toggleGroup(group: PermissionGroup, checked: CheckboxValueType) {
  const ids = new Set(selectedPermissionIds.value)
  for (const permission of group.permissions) {
    if (checked) ids.add(permission.id)
    else ids.delete(permission.id)
  }
  selectedPermissionIds.value = Array.from(ids)
}

function toggleAllGranular(checked: CheckboxValueType) {
  const ids = new Set(selectedPermissionIds.value)
  for (const permission of granularPermissions.value) {
    if (checked) ids.add(permission.id)
    else ids.delete(permission.id)
  }
  selectedPermissionIds.value = Array.from(ids)
}

async function loadRolesAndPermissions() {
  loading.value = true
  try {
    const [roleList, permissionList] = await Promise.all([
      fetchAllPages(fetchRoles),
      fetchPermissions(),
    ])
    roles.value = roleList
    permissions.value = permissionList
    if (!selectedRoleId.value && roles.value.length) {
      selectedRoleId.value = roles.value[0].id
      await loadRolePermissionIds()
    }
  } finally {
    loading.value = false
  }
}

async function loadRolePermissionIds() {
  if (!selectedRoleId.value) return
  loading.value = true
  try {
    const response = await fetchRolePermissionIds(selectedRoleId.value)
    selectedPermissionIds.value = response.permission_ids
  } finally {
    loading.value = false
  }
}

async function save() {
  if (!selectedRoleId.value) {
    ElMessage.error('请选择角色')
    return
  }
  saving.value = true
  try {
    await setRolePermissions(selectedRoleId.value, { permission_ids: selectedPermissionIds.value })
    ElMessage.success('权限已保存')
    await loadRolePermissionIds()
  } finally {
    saving.value = false
  }
}

onMounted(loadRolesAndPermissions)
</script>

<template>
  <div class="permission-page">
    <div class="permission-fixed flex flex-wrap items-center justify-between gap-4">
      <div>
        <h2 class="text-xl font-semibold text-slate-950">权限分配</h2>
        <div class="mt-1 text-sm text-slate-500">{{ permissionSummary }}</div>
      </div>
      <el-button
        type="primary"
        :icon="Check"
        :loading="saving"
        :disabled="!selectedRoleId || selectedRoleIsAdministrator"
        @click="save"
      >
        保存权限
      </el-button>
    </div>

    <section class="permission-fixed rounded-lg border border-slate-200 bg-white p-4">
      <div class="flex flex-wrap items-end gap-5">
        <div class="w-72 max-w-full">
          <div class="mb-2 text-sm font-medium text-slate-700">角色</div>
          <el-select v-model="selectedRoleId" filterable class="w-full" @change="loadRolePermissionIds">
            <el-option v-for="role in roles" :key="role.id" :label="roleLabel(role)" :value="role.id" />
          </el-select>
        </div>
        <div class="ml-auto pb-1">
          <el-checkbox
            :model-value="allGranularSelected"
            :indeterminate="allGranularIndeterminate"
            :disabled="permissionsLocked"
            @change="toggleAllGranular"
          >
            全选普通权限
          </el-checkbox>
        </div>
      </div>
    </section>

    <section
      v-if="wildcardPermission"
      class="permission-fixed flex items-center justify-between gap-4 rounded-lg border border-slate-200 bg-white px-4 py-3"
    >
      <div class="flex min-w-0 items-center gap-3">
        <span class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-slate-100 text-slate-700">
          <el-icon :size="18"><Key /></el-icon>
        </span>
        <div class="min-w-0">
          <div class="font-medium text-slate-950">超级权限</div>
          <div class="truncate font-mono text-xs text-slate-500">* : *</div>
        </div>
      </div>
      <el-checkbox
        :model-value="isPermissionSelected(wildcardPermission.id)"
        :disabled="selectedRoleIsAdministrator"
        @change="(checked: CheckboxValueType) => togglePermission(wildcardPermission!.id, checked)"
      >
        所有权限
      </el-checkbox>
    </section>

    <section class="permission-list rounded-lg border border-slate-200 bg-white">
      <el-table v-loading="loading" :data="permissionGroups" row-key="resource" stripe height="100%">
        <el-table-column label="功能模块" min-width="230" fixed="left">
          <template #default="{ row }">
            <el-checkbox
              :model-value="isGroupSelected(row)"
              :indeterminate="isGroupIndeterminate(row)"
              :disabled="permissionsLocked"
              @change="(checked: CheckboxValueType) => toggleGroup(row, checked)"
            >
              <span class="font-medium text-slate-800">{{ resourceLabel(row.resource) }}</span>
            </el-checkbox>
            <div class="ml-6 mt-0.5 font-mono text-xs text-slate-400">{{ row.resource }}</div>
          </template>
        </el-table-column>

        <el-table-column
          v-for="action in actionColumns"
          :key="action"
          :label="actionLabel(action)"
          min-width="120"
          align="center"
        >
          <template #default="{ row }">
            <template v-if="permissionFor(row, action)">
              <el-tooltip
                :content="permissionFor(row, action)?.description || permissionFor(row, action)?.name"
                placement="top"
              >
                <el-checkbox
                  :model-value="isPermissionSelected(permissionFor(row, action)!.id)"
                  :disabled="permissionsLocked"
                  @change="(checked: CheckboxValueType) => togglePermission(permissionFor(row, action)!.id, checked)"
                />
              </el-tooltip>
            </template>
            <span v-else class="text-slate-300">—</span>
          </template>
        </el-table-column>
      </el-table>
    </section>
  </div>
</template>

<style scoped>
.permission-page {
  display: flex;
  height: 100%;
  min-height: 0;
  flex-direction: column;
  gap: 1rem;
  overflow: hidden;
}

.permission-fixed {
  flex: none;
}

.permission-list {
  min-height: 0;
  flex: 1 1 0%;
  overflow: hidden;
}
</style>
