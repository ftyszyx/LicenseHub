<template>
  <main class="min-h-screen bg-slate-100 px-4 py-10 sm:px-6">
    <section class="mx-auto w-full max-w-md rounded-lg border border-slate-200 bg-white p-6 shadow-sm sm:p-8">
      <div class="mb-7 flex items-center gap-3">
        <span class="flex h-11 w-11 items-center justify-center rounded-lg bg-emerald-600 text-white">
          <el-icon :size="23"><User /></el-icon>
        </span>
        <div>
          <h1 class="text-xl font-semibold text-slate-950">创建账号</h1>
          <p class="mt-1 text-sm text-slate-500">验证邮箱后保存订单、注册码和推广收益。</p>
        </div>
      </div>

      <el-skeleton v-if="settingsLoading" :rows="7" animated />
      <el-result
        v-else-if="!registrationEnabled"
        icon="info"
        title="注册功能暂未开放"
        sub-title="现有账号仍可正常登录和使用。"
      >
        <template #extra>
          <el-button type="primary" @click="router.push(RoutePath.Login)">前往登录</el-button>
          <el-button @click="router.push(RoutePath.Home)">返回商城</el-button>
        </template>
      </el-result>

      <el-form v-else :model="form" label-position="top" @submit.prevent="handleRegister">
        <p v-if="referralCode" class="mb-4 text-sm text-emerald-700">
          检测到推广链接，注册成功后将绑定邀请关系。
        </p>
        <el-form-item label="用户名">
          <el-input v-model="form.username" maxlength="64" autocomplete="username" placeholder="3-64 位字母、数字或 _ - ." />
        </el-form-item>
        <el-form-item label="邮箱">
          <el-input v-model="form.email" maxlength="320" autocomplete="email" placeholder="用于验证和认领历史订单" @input="resetEmailVerification">
            <template #append>
              <el-button :disabled="cooldown > 0" :loading="sendingCode" @click="sendEmailCode">
                {{ cooldown > 0 ? `${cooldown} 秒` : '发送验证码' }}
              </el-button>
            </template>
          </el-input>
        </el-form-item>
        <el-form-item label="邮箱验证码">
          <el-input v-model="form.emailCode" maxlength="6" inputmode="numeric" autocomplete="one-time-code" placeholder="请输入 6 位验证码" />
        </el-form-item>
        <el-form-item label="密码">
          <el-input v-model="form.password" type="password" show-password autocomplete="new-password" placeholder="8-72 个字符" />
        </el-form-item>
        <el-form-item label="确认密码">
          <el-input v-model="form.confirmPassword" type="password" show-password autocomplete="new-password" placeholder="再次输入密码" />
        </el-form-item>
        <el-button native-type="submit" type="primary" class="mt-2 w-full" size="large" :loading="submitting">
          注册并登录
        </el-button>
      </el-form>

      <div v-if="registrationEnabled" class="mt-6 text-center text-sm text-slate-500">
        已有账号？
        <router-link :to="RoutePath.Login" class="font-medium text-blue-600 hover:text-blue-700">去登录</router-link>
      </div>
    </section>

  </main>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { User } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { startEmailVerification, verifyEmailCode } from '@/apis/auth'
import { fetchSiteSettings } from '@/apis/system_settings'
import { useAuthStore } from '@/stores/auth'
import { RoutePath } from '@/types'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const form = reactive({ username: '', email: '', emailCode: '', password: '', confirmPassword: '' })
const settingsLoading = ref(true)
const registrationEnabled = ref(false)
const sendingCode = ref(false)
const submitting = ref(false)
const challengeId = ref('')
const verifiedEmail = ref('')
const referralCode = ref('')
const cooldown = ref(0)
let cooldownTimer: number | undefined
const referralStorageKey = 'licensehub_referral'

function validEmail(value: string) {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value.trim())
}

function resetEmailVerification() {
  if (form.email.trim().toLowerCase() !== verifiedEmail.value) {
    challengeId.value = ''
    form.emailCode = ''
  }
}

function loadReferralCode(attributionDays: number) {
  const queryCode = typeof route.query.ref === 'string'
    ? route.query.ref.trim().toUpperCase()
    : ''
  if (queryCode && queryCode.length <= 32) {
    const expiresAt = Date.now() + attributionDays * 86400000
    localStorage.setItem(referralStorageKey, JSON.stringify({ code: queryCode, expiresAt }))
    referralCode.value = queryCode
    return
  }
  try {
    const saved = JSON.parse(localStorage.getItem(referralStorageKey) || 'null')
    if (saved?.code && saved?.expiresAt > Date.now()) {
      referralCode.value = String(saved.code).trim().toUpperCase()
      return
    }
  } catch {
    // Invalid browser state is treated as no referral.
  }
  localStorage.removeItem(referralStorageKey)
  referralCode.value = ''
}

async function sendEmailCode() {
  if (!validEmail(form.email)) {
    ElMessage.warning('请先填写正确的邮箱地址')
    return
  }
  sendingCode.value = true
  try {
    const result = await startEmailVerification({ email: form.email.trim() })
    challengeId.value = result.challenge_id
    verifiedEmail.value = form.email.trim().toLowerCase()
    startCooldown(result.resend_after_seconds)
    ElMessage.success('验证码已发送，请检查邮箱')
  } finally {
    sendingCode.value = false
  }
}

function startCooldown(seconds: number) {
  if (cooldownTimer) window.clearInterval(cooldownTimer)
  cooldown.value = seconds
  cooldownTimer = window.setInterval(() => {
    cooldown.value = Math.max(0, cooldown.value - 1)
    if (cooldown.value === 0 && cooldownTimer) {
      window.clearInterval(cooldownTimer)
      cooldownTimer = undefined
    }
  }, 1000)
}

async function handleRegister() {
  if (!challengeId.value || form.email.trim().toLowerCase() !== verifiedEmail.value) {
    ElMessage.warning('请先发送邮箱验证码')
    return
  }
  if (!/^\d{6}$/.test(form.emailCode.trim())) {
    ElMessage.warning('请输入 6 位邮箱验证码')
    return
  }
  if (form.password.length < 8 || form.password.length > 72) {
    ElMessage.warning('密码长度必须在 8 到 72 个字符之间')
    return
  }
  if (form.password !== form.confirmPassword) {
    ElMessage.warning('两次输入的密码不一致')
    return
  }
  submitting.value = true
  try {
    const verified = await verifyEmailCode(challengeId.value, form.emailCode.trim())
    await authStore.register({
      username: form.username.trim(),
      email: form.email.trim(),
      password: form.password,
      verification_token: verified.verification_token,
      referral_code: referralCode.value || undefined,
    })
    localStorage.removeItem(referralStorageKey)
    ElMessage.success('注册成功')
    const redirect = typeof route.query.redirect === 'string' && route.query.redirect.startsWith('/')
      ? route.query.redirect
      : RoutePath.UserHome
    await router.push(redirect)
  } finally {
    submitting.value = false
  }
}

onMounted(async () => {
  const email = typeof route.query.email === 'string' ? route.query.email : ''
  if (email) form.email = email
  try {
    const settings = await fetchSiteSettings()
    registrationEnabled.value = Boolean(settings.registration_enabled)
    if (settings.distribution?.enabled && settings.distribution.referrer_binding_enabled) {
      loadReferralCode(settings.distribution.attribution_days)
    } else {
      localStorage.removeItem(referralStorageKey)
    }
  } finally {
    settingsLoading.value = false
  }
})

onBeforeUnmount(() => {
  if (cooldownTimer) window.clearInterval(cooldownTimer)
})
</script>
