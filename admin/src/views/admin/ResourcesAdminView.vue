<template>
  <div class="admin-list-page">
    <el-card class="admin-list-fixed" shadow="hover">
      <div class="flex items-center justify-between gap-3">
        <h2 class="text-xl font-semibold">资源管理</h2>
        <div class="flex items-center gap-2">
          <el-input v-model="query.keyword" clearable class="w-56" placeholder="搜索文件名" @keyup.enter="reload" />
          <el-input v-model="query.resource_type" clearable class="w-44" placeholder="资源类型" @keyup.enter="reload" />
          <el-button type="primary" @click="reload">搜索</el-button>
          <el-button @click="reset">重置</el-button>
          <el-button type="success" @click="openUpload">上传资源</el-button>
        </div>
      </div>
    </el-card>

    <el-card class="admin-list-panel" shadow="never">
      <el-table class="admin-list-table" v-loading="loading" :data="rows" stripe height="100%">
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="resource_type" label="资源类型" width="180" />
        <el-table-column prop="original_name" label="文件名" min-width="240" />
        <el-table-column prop="content_type" label="类型" width="180" />
        <el-table-column label="大小" width="110">
          <template #default="{ row }">{{ formatSize(row.size) }}</template>
        </el-table-column>
        <el-table-column label="存储渠道" min-width="160">
          <template #default="{ row }">{{ row.storage_channel_name || row.storage_channel_id }}</template>
        </el-table-column>
        <el-table-column prop="created_at" label="上传时间" min-width="190" />
        <el-table-column label="操作" width="180" fixed="right" align="right">
          <template #default="{ row }">
            <el-button size="small" type="primary" plain @click="viewResource(row)">查看</el-button>
            <el-button size="small" type="danger" plain @click="removeResource(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="admin-list-footer mt-4 flex justify-end">
        <el-pagination
          background
          layout="total, sizes, prev, pager, next, jumper"
          :page-sizes="[10, 20, 50, 100]"
          :page-size="pageSize"
          :current-page="page"
          :total="total"
          @current-change="handlePageChange"
          @size-change="handleSizeChange"
        />
      </div>
    </el-card>

    <el-dialog v-model="uploadDialog.visible" title="上传资源" width="520px" destroy-on-close>
      <el-form label-width="100px">
        <el-form-item label="资源类型" required>
          <el-input v-model="uploadDialog.resourceType" placeholder="例如 app_image、refund_attachment" />
        </el-form-item>
        <el-form-item label="资源文件" required>
          <el-upload :auto-upload="false" :limit="1" :on-change="handleFileChange" :on-remove="clearFile">
            <el-button>选择文件</el-button>
            <template #tip><div class="el-upload__tip">单个文件最大 20 MB</div></template>
          </el-upload>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="uploadDialog.visible = false">取消</el-button>
        <el-button type="primary" :loading="uploadDialog.submitting" @click="submitUpload">上传</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { ElMessage, ElMessageBox, type UploadFile } from 'element-plus'
import { deleteResource, fetchResourceBlob, fetchResources, uploadResource } from '@/apis/resources'
import type { ListResourcesParams, ResourceModel } from '@/types/resources'

const rows = ref<ResourceModel[]>([])
const loading = ref(false)
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const query = reactive<ListResourcesParams>({ keyword: '', resource_type: '' })
const uploadDialog = reactive({
  visible: false,
  submitting: false,
  resourceType: '',
  file: null as File | null,
})

function formatSize(size: number) {
  if (size < 1024) return `${size} B`
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`
  return `${(size / 1024 / 1024).toFixed(1)} MB`
}

async function reload() {
  loading.value = true
  try {
    const data = await fetchResources({ ...query, page: page.value, page_size: pageSize.value })
    rows.value = data.list
    total.value = data.total
  } finally {
    loading.value = false
  }
}

function reset() {
  query.keyword = ''
  query.resource_type = ''
  page.value = 1
  reload()
}

function handlePageChange(value: number) {
  page.value = value
  reload()
}

function handleSizeChange(value: number) {
  pageSize.value = value
  page.value = 1
  reload()
}

function openUpload() {
  uploadDialog.resourceType = ''
  uploadDialog.file = null
  uploadDialog.visible = true
}

function handleFileChange(file: UploadFile) {
  uploadDialog.file = file.raw || null
}

function clearFile() {
  uploadDialog.file = null
}

async function submitUpload() {
  if (!uploadDialog.resourceType.trim() || !uploadDialog.file) {
    ElMessage.warning('请填写资源类型并选择文件')
    return
  }
  uploadDialog.submitting = true
  try {
    await uploadResource(uploadDialog.resourceType.trim(), uploadDialog.file)
    ElMessage.success('上传成功')
    uploadDialog.visible = false
    await reload()
  } finally {
    uploadDialog.submitting = false
  }
}

async function viewResource(row: ResourceModel) {
  const blob = await fetchResourceBlob(row.id)
  const url = URL.createObjectURL(blob)
  window.open(url, '_blank', 'noopener,noreferrer')
  window.setTimeout(() => URL.revokeObjectURL(url), 60_000)
}

async function removeResource(row: ResourceModel) {
  await ElMessageBox.confirm(`确定删除资源“${row.original_name}”吗？`, '确认删除', { type: 'warning' })
  await deleteResource(row.id)
  ElMessage.success('删除成功')
  await reload()
}

onMounted(reload)
</script>
