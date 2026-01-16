<template>
  <div class="space-y-4">
    <el-card shadow="hover">
      <div class="flex items-center justify-between">
        <h2 class="text-xl font-semibold">{{ $t('apps.title') }}</h2>
        <div class="flex items-center gap-2">
          <el-input v-model="query.name" :placeholder="$t('common.search_by_name')" clearable class="w-64" />
          <el-button type="primary" @click="reload">
            <el-icon class="mr-1"><Search /></el-icon>
            {{ $t("common.search") }}
          </el-button>
          <el-button @click="resetFilters">
            <el-icon class="mr-1"><Refresh /></el-icon>
            {{ $t("common.reset") }}
          </el-button>
          <el-button type="success" @click="openCreate">
            <el-icon class="mr-1"><Plus /></el-icon>
            {{ $t("common.new") }}
          </el-button>
        </div>
      </div>
    </el-card>

    <el-card shadow="never">
      <el-table :data="apps" stripe size="large" style="width: 100%">
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column :label="$t('apps.name')" min-width="220">
          <template #default="{ row }">
            <div class="flex flex-col leading-tight">
              <span class="font-medium">{{ row.name }}</span>
              <span class="text-gray-500 text-xs">{{ row.app_id }}</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column :label="$t('apps.version')" min-width="160">
          <template #default="{ row }">
            <el-tag type="info" effect="light">{{ row.app_vername }}</el-tag>
            <span class="ml-2 text-gray-500">(#{{ row.app_vercode }})</span>
          </template>
        </el-table-column>
        <el-table-column :label="$t('apps.valid_key')" min-width="260">
          <template #default="{ row }">
            <div class="flex items-center gap-2">
              <span class="text-gray-600 break-all">{{ row.app_valid_key }}</span>
              <el-button size="small" @click="copyKey(row.app_valid_key)">Copy</el-button>
            </div>
          </template>
        </el-table-column>
        <el-table-column :label="$t('apps.trial_days')" width="120">
          <template #default="{ row }">{{ row.trial_days }}</template>
        </el-table-column>
        <el-table-column :label="$t('apps.trial_num')" width="120">
          <template #default="{ row }">{{ row.trial_num }}</template>
        </el-table-column>
        <el-table-column :label="$t('apps.code_type')" width="120">
          <template #default="{ row }">
            <span v-if="row.code_type === RegCodeType.Time">{{ $t('reg_codes.type_time') }}</span>
            <span v-else-if="row.code_type === RegCodeType.Count">{{ $t('reg_codes.type_count') }}</span>
            <span v-else>{{ row.code_type }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="status" :label="$t('apps.status')" width="100">
          <template #default="scope">
            <el-tag :type="scope.row.status === 1 ? 'success' : 'info'">{{
              scope.row.status === 1 ? $t("apps.enabled") : $t("apps.disabled")
            }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="$t('apps.links')" min-width="200">
          <template #default="{ row }">
            <el-link :href="row.app_download_url" target="_blank" type="primary" underline="never" class="mr-3">
              <el-icon class="mr-1"><Download /></el-icon>{{ $t("apps.download") }}
            </el-link>
            <el-link :href="row.app_res_url" target="_blank" type="primary" underline="never">
              <el-icon class="mr-1"><Link /></el-icon>{{ $t("apps.resource") }}
            </el-link>
          </template>
        </el-table-column>
        <el-table-column :label="$t('apps.update_info')" min-width="200">
          <template #default="{ row }">
            <el-tooltip :content="row.app_update_info || '-'" placement="top" effect="dark">
              <span class="truncate block max-w-[320px] text-gray-600">{{ row.app_update_info || "-" }}</span>
            </el-tooltip>
          </template>
        </el-table-column>
        <el-table-column :label="$t('apps.created')" min-width="180">
          <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
        </el-table-column>
        <el-table-column :label="$t('apps.updated')" min-width="180">
          <template #default="{ row }">{{ formatTime(row.updated_at) }}</template>
        </el-table-column>
        <el-table-column :label="$t('common.actions')" width="200" fixed="right">
          <template #default="scope">
            <el-button size="small" @click="openEdit(scope.row)">
              <el-icon class="mr-1"><Edit /></el-icon>
              {{ $t("common.edit") }}
            </el-button>
            <el-button size="small" type="danger" @click="confirmDelete(scope.row)">
              <el-icon class="mr-1"><Delete /></el-icon>
              {{ $t("common.delete") }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <div class="flex justify-end mt-4">
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

    <el-dialog v-model="dialog.visible" :title="dialog.mode === 'create' ? 'Create App' : 'Edit App'" width="720px">
      <el-form :model="form" label-width="140px" class="pr-4" ref="formRef" :rules="rules">
        <el-form-item :label="$t('apps.name')" prop="name">
          <el-input v-model="form.name" />
        </el-form-item>
        <el-form-item :label="$t('apps.app_id')" prop="app_id">
          <el-input v-model="form.app_id">
            <template #append>
              <el-button @click="genAppId">{{ $t('common.generate') }}</el-button>
            </template>
          </el-input>
        </el-form-item>
        <el-form-item :label="$t('apps.version_name')" prop="app_vername">
          <el-input v-model="form.app_vername" />
        </el-form-item>
        <el-form-item :label="$t('apps.version_code')" prop="app_vercode">
          <el-input-number v-model="form.app_vercode" :min="0" />
        </el-form-item>
        <el-form-item :label="$t('apps.download_url')" prop="app_download_url">
          <el-input v-model="form.app_download_url" />
        </el-form-item>
        <el-form-item :label="$t('apps.resource_url')" prop="app_res_url">
          <el-input v-model="form.app_res_url" />
        </el-form-item>
        <el-form-item :label="$t('apps.update_info')" prop="app_update_info">
          <el-input v-model="form.app_update_info" type="textarea" :rows="3" />
        </el-form-item>
        <el-form-item :label="$t('apps.valid_key')" prop="app_valid_key">
          <el-input v-model="(form as any).app_valid_key">
            <template #append>
              <el-button @click="genAppKey">{{ $t('common.generate') }}</el-button>
            </template>
          </el-input>
        </el-form-item>
        <el-form-item :label="$t('apps.code_type')" prop="code_type">
          <el-select v-model.number="(form as any).code_type" style="width: 160px" @change="onCodeTypeChange">
            <el-option :label="$t('reg_codes.type_time')" :value="RegCodeType.Time" />
            <el-option :label="$t('reg_codes.type_count')" :value="RegCodeType.Count" />
          </el-select>
        </el-form-item>
        <el-form-item v-if="(form as any).code_type === RegCodeType.Time" :label="$t('apps.trial_days')" prop="trial_days">
          <el-input-number v-model.number="(form as any).trial_days" :min="0" />
        </el-form-item>
        <el-form-item v-if="(form as any).code_type === RegCodeType.Count" :label="$t('apps.trial_num')" prop="trial_num">
          <el-input-number v-model.number="(form as any).trial_num" :min="0" />
        </el-form-item>
        <el-form-item :label="$t('apps.sort_order')" prop="sort_order">
          <el-input-number v-model="form.sort_order" :min="0" />
        </el-form-item>
        <el-form-item :label="$t('apps.status')" prop="status">
          <el-select v-model="form.status" style="width: 160px">
            <el-option :label="$t('apps.enabled')" :value="1" />
            <el-option :label="$t('apps.disabled')" :value="0" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="dialog.visible = false">{{ $t('common.cancel') }}</el-button>
          <el-button type="primary" @click="submitForm(formRef)">{{ $t('common.confirm') }}</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { fetchApps, createApp, updateApp, deleteApp } from "@/apis/apps";
import type { AppModel, AddAppReq, UpdateAppReq } from "@/types/apps";
import { ElMessage, ElMessageBox } from "element-plus";
import type { FormInstance, FormRules } from "element-plus";
import { useI18n } from 'vue-i18n'
import { RegCodeType } from '@/types/reg_codes'

const apps = ref<AppModel[]>([]);
const page = ref(1);
const pageSize = ref(20);
const total = ref(0);

const query = reactive({
  name: "" as string | undefined,
});

const reload = async () => {
  const data = await fetchApps({ page: page.value, page_size: pageSize.value, name: query.name || undefined });
  apps.value = data.list;
  total.value = data.total;
};

const handlePageChange = async (p: number) => {
  page.value = p;
  await reload();
};

const handleSizeChange = async (s: number) => {
  pageSize.value = s;
  page.value = 1;
  await reload();
};

const resetFilters = async () => {
  query.name = "";
  page.value = 1;
  await reload();
};
onMounted(reload);
const dialog = reactive({ visible: false, mode: "create" as "create" | "edit", editingId: undefined as number | undefined });
const emptyForm: AddAppReq = {
  name: "",
  app_id: "",
  app_vername: "",
  app_vercode: 0,
  app_download_url: "",
  app_res_url: "",
  app_update_info: "",
  code_type: 0,
  app_valid_key: "",
  trial_days: 0,
  trial_num: 0,
  sort_order: 0,
  status: 1,
};
const form = reactive<AddAppReq>({ ...emptyForm });
const formRef = ref<FormInstance>();
const { t } = useI18n()
const rules = reactive<FormRules<AddAppReq>>({
  name: [{ required: true, message: t("apps.input_name") }],
  app_id: [{ required: true, message: t("apps.input_app_id") }],
  app_vername: [{ required: true, message: t("apps.input_version_name") }],
  app_valid_key: [{ required: true, message: t("apps.input_valid_key") }],
  app_vercode: [{ required: true, message: t("apps.input_version_code") }],
});

const onCodeTypeChange = () => {
  if ((form as any).code_type === RegCodeType.Time) {
    ;(form as any).trial_num = 0
  } else if ((form as any).code_type === RegCodeType.Count) {
    ;(form as any).trial_days = 0
  }
}

const openCreate = () => {
  Object.assign(form, emptyForm);
  onCodeTypeChange();
  dialog.mode = "create";
  dialog.editingId = undefined;
  dialog.visible = true;
};

const openEdit = (row: AppModel) => {
  dialog.mode = "edit";
  dialog.editingId = row.id;
  Object.assign(form, {
    name: row.name,
    app_id: row.app_id,
    app_vername: row.app_vername,
    app_vercode: row.app_vercode,
    app_download_url: row.app_download_url,
    app_res_url: row.app_res_url,
    app_update_info: row.app_update_info || "",
    code_type: (row as any).code_type ?? 0,
    app_valid_key: row.app_valid_key || "",
    trial_days: row.trial_days ?? 0,
    trial_num: (row as any).trial_num ?? 0,
    sort_order: row.sort_order,
    status: row.status,
  });
  onCodeTypeChange();
  dialog.visible = true;
};

const submitForm = async (formRef: FormInstance|undefined) => {
  //validate form
  if (!formRef) return; 
  const valid = await formRef.validate();
  if (!valid) {
    ElMessage.error("Please check the form");
    return;
  }
  if (dialog.mode === "create") {
    await createApp(form);
  } else if (dialog.editingId != null) {
    const payload: UpdateAppReq = { ...form };
    await updateApp(dialog.editingId, payload);
  }
  dialog.visible = false;
  await reload();
  ElMessage.success("Saved");
};

function genAppKey(){
  const ts = Date.now().toString(36)
  const rand = Math.random().toString(36).slice(2)
  ;(form as any).app_valid_key = `${ts}-${rand}`
}

function genAppId(){
  const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'
  const len = 12
  if (typeof crypto !== 'undefined' && 'getRandomValues' in crypto) {
    const array = new Uint8Array(len)
    crypto.getRandomValues(array)
    form.app_id = Array.from(array, n => alphabet[n % alphabet.length]).join('')
  } else {
    let s = ''
    for (let i = 0; i < len; i++) {
      s += alphabet[Math.floor(Math.random() * alphabet.length)]
    }
    form.app_id = s
  }
}

const confirmDelete = async (row: AppModel) => {
  try {
    await ElMessageBox.confirm(t("apps.delete_confirm", { name: row.name }), t("common.confirm"), {
      type: "warning",
      confirmButtonText: t("common.delete"),
      cancelButtonText: t("common.cancel"),
    });
    await deleteApp(row.id);
    ElMessage.success("Deleted");
    await reload();
  } catch (_) {
    // cancel
  }
};

const formatTime = (iso: string) => {
  if (!iso) return "-";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
};

async function copyKey(key: string){
  try {
    await navigator.clipboard.writeText(key || '')
    ElMessage.success('Copied')
  } catch (_) {
    ElMessage.error('Copy failed')
  }
}
</script>

<style scoped></style>
