<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox, type FormInstance, type FormRules } from 'element-plus'
import { formatTime } from '@/utils'
import { fetchRoles, createRole, updateRole, deleteRole } from '@/apis/roles'
import { useI18n } from 'vue-i18n'
import type { RoleInfo } from '@/types'
const rows = ref<RoleInfo[]>([])
const selectedIds = ref<number[]>([])
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const { t } = useI18n()

const query = reactive({ name: '' as string | undefined })

async function reload() { const data = await fetchRoles({ page: page.value, page_size: pageSize.value, name: query.name || undefined }); rows.value = data.list; total.value = data.total }
function resetFilters() { query.name = ''; page.value = 1; reload() }
function onSelChange(arr: RoleInfo[]) { selectedIds.value = arr.map(it => it.id) }
function handlePageChange(p: number) { page.value = p; reload() }
function handleSizeChange(s: number) { pageSize.value = s; page.value = 1; reload() }

const dialog = reactive({ visible: false, mode: 'create' as 'create' | 'edit', editingId: undefined as number | undefined })
const formRef = ref<FormInstance>()
const form = reactive<{ name: string; description?: string | null }>({ name: '', description: '' })
const rules = reactive<FormRules>({ name: [{ required: true, message: 'Name required' }] })

function openCreate() { dialog.mode = 'create'; dialog.editingId = undefined; form.name = ''; form.description = ''; dialog.visible = true }
function openEdit(row: RoleInfo) { dialog.mode = 'edit'; dialog.editingId = row.id; form.name = row.name; form.description = row.description || ''; dialog.visible = true }

async function submit() {
  const valid = await formRef.value?.validate(); if (!valid) { ElMessage.error(t('common.please_check_form') as string); return }
  if (dialog.mode === 'create') { await createRole({ name: form.name, description: form.description || undefined }); ElMessage.success(t('common.created') as string) }
  else if (dialog.editingId != null) { await updateRole(dialog.editingId, { name: form.name, description: form.description || undefined }); ElMessage.success(t('common.save') as string) }
  dialog.visible = false; await reload()
}

async function del(id: number) { await ElMessageBox.confirm(t('common.delete_confirm', { name: rows.value.find(it => it.id === id)?.name || '' }), t('common.confirm'), { type: 'warning' }); await deleteRole(id); ElMessage.success(t('common.deleted') as string); reload() }

onMounted(reload)
</script>

<template>
  <div class="admin-list-page">
    <el-card class="admin-list-fixed" shadow="hover">
      <div class="flex items-center gap-2">
        <el-input v-model="query.name" :placeholder="$t('common.search_by_name')" clearable class="w-64" />
        <el-button type="primary" @click="reload">{{ $t('common.search') }}</el-button>
        <el-button @click="resetFilters">{{ $t('common.reset') }}</el-button>
        <el-button type="success" @click="openCreate">{{ $t('common.new') }}</el-button>
      </div>
    </el-card>

    <el-card class="admin-list-panel" shadow="never">
      <el-table class="admin-list-table" :data="rows" stripe size="large" height="100%" @selection-change="onSelChange">
        <el-table-column type="selection" width="50" />
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column :label="$t('common.name')" min-width="160">
          <template #default="{ row }">
            <div class="flex items-center gap-2">
              <span class="text-gray-800 break-all">{{ row.name }}</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column :label="$t('common.remark')" min-width="200">
          <template #default="{ row }">{{ row.description }}</template>
        </el-table-column>
        <el-table-column :label="$t('common.created')" min-width="180">
          <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
        </el-table-column>
        <el-table-column :label="$t('common.actions')" width="200" fixed="right">
          <template #default="{ row }">
            <el-button size="small" @click="openEdit(row)">{{ $t('common.edit') }}</el-button>
            <el-button size="small" type="danger" @click="del(row.id)">{{ $t('common.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="admin-list-footer mt-4 flex justify-end">
        <el-pagination background layout="total, sizes, prev, pager, next, jumper" :page-sizes="[10, 20, 50, 100]"
          :page-size="pageSize" :current-page="page" :total="total" @current-change="handlePageChange"
          @size-change="handleSizeChange" />
      </div>
    </el-card>

    <el-dialog v-model="dialog.visible" :title="dialog.mode === 'create' ? $t('common.create') : $t('common.edit')" width="520px">
      <el-form label-width="140px" ref="formRef" :model="form" :rules="rules">
        <el-form-item :label="$t('common.name')" prop="name"><el-input v-model="form.name" /></el-form-item>
        <el-form-item :label="$t('common.remark')" prop="description"><el-input v-model="form.description" /></el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialog.visible = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submit">{{ $t('common.confirm') }}</el-button>
      </template>
    </el-dialog>
  </div>
  
</template>

<style scoped></style>


