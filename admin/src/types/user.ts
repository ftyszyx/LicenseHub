import type { ListParamsReq } from "./api"

export interface ChangePasswordPayload {
  old_password: string
  new_password: string
}

export interface UserModel {
  id: number
  username: string
  email?: string | null
  email_verified_at?: string | null
  referral_code: string
  referrer_user_id?: number | null
  referrer_bound_at?: string | null
  registered_referral_code?: string | null
  commission_rate_bps?: number | null
  created_at: string
  updated_at: string
}

export interface UserWithRoles {
  user: UserModel
  role_ids: number[]
}

export type CreateUserReq = { username: string; password: string; role_ids?: number[] }
export type UpdateUserReq = { username?: string; password?: string; role_ids?: number[]; commission_rate_bps?: number | null }
export type ListUsersParams = { username?: string; id?: number } & ListParamsReq
