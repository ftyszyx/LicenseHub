import { defineStore } from 'pinia'
import type { AuthPayload, RegisterPayload } from '@/types'
import { computed, ref } from 'vue'
import { sentLogin, sentRegister } from '@/apis'

type JwtPayload = {
  exp?: number
  role_ids?: number[]
}

const ADMIN_ROLE_ID = 1

function decodeBase64Url(value: string) {
  const normalized = value.replace(/-/g, '+').replace(/_/g, '/')
  const padded = normalized.padEnd(Math.ceil(normalized.length / 4) * 4, '=')
  return atob(padded)
}

function isTokenUsable(token: string | null) {
  if (!token) return false
  const [, payload] = token.split('.')
  if (!payload) return false

  try {
    const claims = JSON.parse(decodeBase64Url(payload)) as JwtPayload
    return typeof claims.exp === 'number' && claims.exp * 1000 > Date.now()
  } catch {
    return false
  }
}

function decodeToken(token: string | null): JwtPayload | null {
  if (!token) return null
  const [, payload] = token.split('.')
  if (!payload) return null
  try {
    return JSON.parse(decodeBase64Url(payload)) as JwtPayload
  } catch {
    return null
  }
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref(localStorage.getItem('token'))
  const isAuthenticated = computed(() => isTokenUsable(token.value))
  const roleIds = computed(() => decodeToken(token.value)?.role_ids || [])
  const isAdmin = computed(() => roleIds.value.includes(ADMIN_ROLE_ID))

  if (token.value && !isTokenUsable(token.value)) {
    localStorage.removeItem('token')
    token.value = null
  }

  function hasValidSession() {
    if (isTokenUsable(token.value)) return true
    clearToken()
    return false
  }

  function setToken(newToken: string) {
    token.value = newToken
    localStorage.setItem('token', newToken)
  }

  function clearToken() {
    token.value = null
    localStorage.removeItem('token')
  }

  async function login(payload: AuthPayload) {
    const response = await sentLogin(payload)
    setToken(response.token)
  }

  async function register(payload: RegisterPayload) {
    const response = await sentRegister(payload)
    setToken(response.token)
  }

  function logout() {
    clearToken()
  }

  return { token, isAuthenticated, roleIds, isAdmin, hasValidSession, login, logout, register, clearToken }
})
