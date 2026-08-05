

import request from '@/utils/request'
import type { PagingResponse } from '@/types'
import type { ListUsersParams, UserWithRoles, CreateUserReq, UpdateUserReq } from '@/types/user'

export async function fetchUsers(params: ListUsersParams): Promise<PagingResponse<UserWithRoles>> {
  return (await request.get('/admin/users/list', { params })).data
}

export async function createUser(payload: CreateUserReq): Promise<UserWithRoles> {
  return (await request.post('/admin/users', payload)).data
}

export async function updateUser(id: number, payload: UpdateUserReq): Promise<UserWithRoles> {
  return (await request.put(`/admin/users/${id}`, payload)).data
}

export async function deleteUser(id: number): Promise<void> {
  return (await request.delete(`/admin/users/${id}`)).data
}

export async function resetReferralCode(id: number): Promise<UserWithRoles> {
  return (await request.post(`/admin/users/${id}/referral-code/reset`)).data
}
