<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox, type FormInstance, type FormRules } from 'element-plus'
import { formatTime } from '@/utils'
import { fetchUsers, createUser, updateUser, deleteUser, resetReferralCode } from '@/apis/users'
import { fetchRoles } from '@/apis/roles'
import { useI18n } from 'vue-i18n'
import type { UserWithRoles } from '@/types/user'
import type { RoleInfo } from '@/types'
const rows = ref<UserWithRoles[]>([])
const roles = ref<RoleInfo[]>([])
const selectedIds = ref<number[]>([])

const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const { t } = useI18n()
const query = reactive({ username: '' as string | undefined })
async function reload() {
  const data = await fetchUsers({ page: page.value, page_size: pageSize.value, username: query.username || undefined });
  rows.value = data.list;
  total.value = data.total
}
async function reloadRoles() {
  const data = await fetchRoles({ page: 1, page_size: 1000 });
  roles.value = data.list
}
function resetFilters() { query.username = ''; page.value = 1; reload() }
function onSelChange(arr: UserWithRoles[]) { selectedIds.value = arr.map(it => it.user.id) }
function handlePageChange(p: number) { page.value = p; reload() }
function handleSizeChange(s: number) { pageSize.value = s; page.value = 1; reload() }
const dialog = reactive({ visible: false, mode: 'create' as 'create' | 'edit', editingId: undefined as number | undefined })
const formRef = ref<FormInstance>()
const form = reactive<{ username: string; password?: string; role_ids?: number[]; commission_rate_percent?: number | null }>({ username: '', password: '', role_ids: [], commission_rate_percent: null })
const rules = reactive<FormRules>({ username: [{ required: true, message: 'Username required' }] })
function openCreate() { dialog.mode = 'create'; dialog.editingId = undefined; form.username = ''; form.password = ''; form.role_ids = []; form.commission_rate_percent = null; dialog.visible = true }
function openEdit(row: UserWithRoles) { dialog.mode = 'edit'; dialog.editingId = row.user.id; form.username = row.user.username; form.password = ''; form.role_ids = [...(row.role_ids || [])]; form.commission_rate_percent = row.user.commission_rate_bps == null ? null : row.user.commission_rate_bps / 100; dialog.visible = true }

function roleNames(roleIds: number[]) {
  const names = roleIds
    .map(id => roles.value.find(r => r.id === id)?.name)
    .filter((v): v is string => Boolean(v))
  return names.join(', ')
}
async function submit() {
  const valid = await formRef.value?.validate();
  if (!valid) { ElMessage.error(t('common.please_check_form') as string); return }
  if (dialog.mode === 'create') { await createUser({ username: form.username, password: form.password || '', role_ids: form.role_ids }); ElMessage.success(t('common.created') as string) }
  else if (dialog.editingId != null) { await updateUser(dialog.editingId, { username: form.username, role_ids: form.role_ids, commission_rate_bps: form.commission_rate_percent == null ? null : Math.round(form.commission_rate_percent * 100) }); ElMessage.success(t('common.save') as string) }
  dialog.visible = false; await reload()
}
async function del(id: number) { await ElMessageBox.confirm(t('common.delete_confirm', { name: rows.value.find(it => it.user.id === id)?.user.username || '' }), t('common.confirm'), { type: 'warning' }); await deleteUser(id); ElMessage.success(t('common.deleted') as string); reload() }
async function resetCode(id: number) { await resetReferralCode(id); ElMessage.success('推广码已重置'); await reload() }
onMounted(() => { reload(); reloadRoles() })
</script>

<template>
  <div class="space-y-4">
    <el-card shadow="hover">
      <div class="flex items-center gap-2">
        <el-input v-model="query.username" :placeholder="$t('common.search_by_name')" clearable class="w-64" />
        <el-button type="primary" @click="reload">{{ $t('common.search') }}</el-button>
        <el-button @click="resetFilters">{{ $t('common.reset') }}</el-button>
        <el-button type="success" @click="openCreate">{{ $t('common.new') }}</el-button>
      </div>
    </el-card>

    <el-card shadow="never">
      <el-table :data="rows" stripe size="large" style="width: 100%" @selection-change="onSelChange">
        <el-table-column type="selection" width="50" />
        <el-table-column prop="user.id" label="ID" width="80" />
        <el-table-column :label="$t('auth.username')" min-width="160">
          <template #default="{ row }">
            <div class="flex items-center gap-2">
              <span class="text-gray-800 break-all">{{ row.user.username }}</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column :label="$t('menu.roles')" min-width="140">
          <template #default="{ row }">{{ roleNames(row.role_ids || []) }}</template>
        </el-table-column>
        <el-table-column label="邮箱" min-width="210">
          <template #default="{ row }">
            <div v-if="row.user.email" class="flex items-center gap-2">
              <span class="break-all text-gray-700">{{ row.user.email }}</span>
              <el-tag v-if="row.user.email_verified_at" size="small" type="success">已验证</el-tag>
            </div>
            <span v-else class="text-gray-400">未绑定</span>
          </template>
        </el-table-column>
        <el-table-column label="推广码" min-width="150" prop="user.referral_code" />
        <el-table-column label="佣金比例" width="110">
          <template #default="{ row }">{{ row.user.commission_rate_bps == null ? '默认' : `${(row.user.commission_rate_bps / 100).toFixed(2)}%` }}</template>
        </el-table-column>
        <el-table-column :label="$t('orders.created')" min-width="180">
          <template #default="{ row }">{{ formatTime(row.user.created_at) }}</template>
        </el-table-column>
        <el-table-column :label="$t('common.actions')" width="280" fixed="right">
          <template #default="{ row }">
            <el-button size="small" @click="openEdit(row)">{{ $t('common.edit') }}</el-button>
            <el-button size="small" @click="resetCode(row.user.id)">重置推广码</el-button>
            <el-button size="small" type="danger" @click="del(row.user.id)">{{ $t('common.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>

      <div class="flex justify-end mt-4">
        <el-pagination background layout="total, sizes, prev, pager, next, jumper" :page-sizes="[10, 20, 50, 100]"
          :page-size="pageSize" :current-page="page" :total="total" @current-change="handlePageChange"
          @size-change="handleSizeChange" />
      </div>
    </el-card>

    <el-dialog v-model="dialog.visible" :title="dialog.mode === 'create' ? $t('common.create') : $t('common.edit')"
      width="520px">
      <el-form label-width="140px" ref="formRef" :model="form" :rules="rules">
        <el-form-item :label="$t('auth.username')" prop="username"><el-input v-model="form.username" /></el-form-item>
        <el-form-item :label="$t('menu.roles')" prop="role_ids">
          <el-select v-model="form.role_ids" multiple class="w-full">
            <el-option v-for="r in roles" :key="r.id" :label="r.name" :value="r.id" />
          </el-select>
        </el-form-item>
        <el-form-item v-if="dialog.mode === 'create'" :label="$t('auth.password')" prop="password"><el-input
            v-model="form.password" type="password" /></el-form-item>
        <el-form-item v-if="dialog.mode === 'edit'" label="个人佣金比例">
          <el-input-number v-model="form.commission_rate_percent" :min="0" :max="100" :precision="2" clearable />
          <span class="ml-2 text-sm text-gray-500">%，留空使用系统默认值</span>
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="dialog.visible = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submit">{{ $t('common.confirm') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped></style>
